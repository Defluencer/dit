use crate::utils::dag_nodes::{
    ipfs_dag_get_node_async, ipfs_dag_put_node_async, search_keypairs, update_ipns,
};

use std::convert::TryFrom;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::feed::Feed;
use linked_data::video::{DayNode, HourNode, MinuteNode, VideoMetadata};

use cid::Cid;

use structopt::StructOpt;

pub const FEED_KEY: &str = "feed";

#[derive(Debug, StructOpt)]
pub struct ContentFeed {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Publish new content to your feed.
    Add(AddContent),

    /// Update content on your feed.
    Update(UpdateContent),

    /// Delete content from your feed.
    Delete(DeleteContent),
}

pub async fn content_feed_cli(cli: ContentFeed) {
    let res = match cli.cmd {
        Command::Add(add) => match add {
            AddContent::MicroBlog(blog) => add_micro_blog(blog).await,
            AddContent::Blog(blog) => add_blog(blog).await,
            AddContent::Video(video) => add_video(video).await,
        },
        Command::Update(update) => match update {
            UpdateContent::MicroBlog(blog) => update_micro_blog(blog).await,
            UpdateContent::Blog(blog) => update_blog(blog).await,
            UpdateContent::Video(video) => update_video(video).await,
        },
        Command::Delete(delete) => delete_content(delete).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {}", e);
    }
}

#[derive(Debug, StructOpt)]
enum AddContent {
    /// Create new micro post.
    MicroBlog(AddMicroPost),

    /// Create new blog post.
    Blog(AddPost),

    /// Create new video post.
    Video(AddVideo),
}

async fn add_content_to_feed(ipfs: &IpfsClient, new_cid: Cid) -> Result<usize, Error> {
    println!("Updating Content Feed...");

    let mut feed = get_feed(ipfs).await?;

    ipfs.pin_add(&new_cid.to_string(), true).await?;

    feed.content.push(new_cid.into());

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    Ok(feed.content.len() - 1)
}

#[derive(Debug, StructOpt)]
pub struct AddMicroPost {
    /// The micro post content.
    #[structopt(short, long)]
    content: String,
}

async fn add_micro_blog(command: AddMicroPost) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let AddMicroPost { content } = command;

    let metadata = MicroPost::create(content);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    println!("New Post CID => {}", &new_cid.to_string());

    let index = add_content_to_feed(&ipfs, new_cid).await?;

    println!("✅ Weblog Post Added In Content Feed At Index {}", index);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct AddPost {
    /// The blog post title.
    #[structopt(short, long)]
    title: String,

    /// The post thumbnail image CID.
    #[structopt(short, long)]
    image: Cid,

    /// The markdown file CID.
    #[structopt(short, long)]
    content: Cid,
}

async fn add_blog(command: AddPost) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let AddPost {
        title,
        image,
        content,
    } = command;

    let metadata = FullPost::create(title, image, content);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    println!("New Post CID => {}", &new_cid.to_string());

    let index = add_content_to_feed(&ipfs, new_cid).await?;

    println!("✅ Weblog Post Added In Content Feed At Index {}", index);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct AddVideo {
    /// The new video title.
    #[structopt(short, long)]
    title: String,

    /// The new video thumbnail image CID.
    #[structopt(short, long)]
    image: Cid,

    /// The new video timecode CID.
    #[structopt(short, long)]
    video: Cid,
}

async fn add_video(command: AddVideo) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let AddVideo {
        title,
        image,
        video,
    } = command;

    let duration = get_video_duration(&ipfs, &video).await?;

    let metadata = VideoMetadata::create(title, duration, image, video);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    println!("New Post CID => {}", &new_cid.to_string());

    let index = add_content_to_feed(&ipfs, new_cid).await?;

    println!("✅ Video Post Added In Content Feed At Index {}", index);

    Ok(())
}

#[derive(Debug, StructOpt)]
enum UpdateContent {
    /// Update micro blog post.
    MicroBlog(UpdateMicroPost),

    /// Update blog post.
    Blog(UpdatePost),

    /// Update video post.
    Video(UpdateVideo),
}

#[derive(Debug, StructOpt)]
pub struct UpdateMicroPost {
    /// The content feed index of the post to update.
    #[structopt(long)]
    index: usize,

    /// The new content.
    #[structopt(short, long)]
    content: Option<String>,
}

async fn update_micro_blog(command: UpdateMicroPost) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let mut feed = get_feed(&ipfs).await?;

    let UpdateMicroPost { index, content } = command;

    let old_cid = match feed.content.get(index) {
        Some(mt) => mt.link,
        None => return Err(Error::Uncategorized("Index Not Found".into())),
    };

    ipfs.pin_rm(&old_cid.to_string(), true).await?;

    let mut metadata: MicroPost = ipfs_dag_get_node_async(&ipfs, &old_cid.to_string()).await?;

    metadata.update(content);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    println!("New Post CID => {}", &new_cid.to_string());

    println!("Updating Content Feed...");

    ipfs.pin_add(&new_cid.to_string(), true).await?;

    feed.content[index] = new_cid.into();

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!("✅ Weblog Post Updated In Content Feed At Index {}", index);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UpdatePost {
    /// The content feed index of the post to update.
    #[structopt(long)]
    index: usize,

    /// The new title.
    #[structopt(short, long)]
    title: Option<String>,

    /// The new thumbnail image CID.
    #[structopt(short, long)]
    image: Option<Cid>,

    /// The new makdown file CID.
    #[structopt(short, long)]
    content: Option<Cid>,
}

async fn update_blog(command: UpdatePost) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let mut feed = get_feed(&ipfs).await?;

    let UpdatePost {
        index,
        title,
        image,
        content,
    } = command;

    let old_cid = match feed.content.get(index) {
        Some(mt) => mt.link,
        None => return Err(Error::Uncategorized("Index Not Found".into())),
    };

    ipfs.pin_rm(&old_cid.to_string(), true).await?;

    let mut metadata: FullPost = ipfs_dag_get_node_async(&ipfs, &old_cid.to_string()).await?;

    metadata.update(title, image, content);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    println!("New Post CID => {}", &new_cid.to_string());

    println!("Updating Content Feed...");

    ipfs.pin_add(&new_cid.to_string(), true).await?;

    feed.content[index] = new_cid.into();

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!("✅ Weblog Post Updated In Content Feed At Index {}", index);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UpdateVideo {
    /// The content feed index of the video to update.
    #[structopt(long)]
    index: usize,

    /// The new video title.
    #[structopt(short, long)]
    title: Option<String>,

    /// The new video thumbnail image CID.
    #[structopt(short, long)]
    image: Option<Cid>,

    /// The new video timecode CID.
    #[structopt(short, long)]
    video: Option<Cid>,
}

async fn update_video(command: UpdateVideo) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let mut feed = get_feed(&ipfs).await?;

    let UpdateVideo {
        index,
        title,
        image,
        video,
    } = command;

    let old_cid = match feed.content.get(index) {
        Some(mt) => mt.link,
        None => return Err(Error::Uncategorized("Index Not Found".into())),
    };

    let mut metadata: VideoMetadata = ipfs_dag_get_node_async(&ipfs, &old_cid.to_string()).await?;

    let duration = match video {
        Some(cid) => Some(get_video_duration(&ipfs, &cid).await?),
        None => None,
    };

    metadata.update(title, image, video, duration);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    println!("New Post CID => {}", &new_cid.to_string());

    println!("Updating Content Feed...");

    feed.content[index] = new_cid.into();

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!("✅ Video Post Updated In Content Feed At Index {}", index);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct DeleteContent {
    /// The index of the content to delete.
    #[structopt(short, long)]
    index: usize,
}

async fn delete_content(command: DeleteContent) -> Result<(), Error> {
    println!("Deleting Content...");
    let ipfs = IpfsClient::default();

    let DeleteContent { index } = command;

    let mut feed = get_feed(&ipfs).await?;

    if index >= feed.content.len() {
        return Err(Error::Uncategorized("Index Not Found".into()));
    }

    let link = feed.content.remove(index);

    ipfs.pin_rm(&link.link.to_string(), true).await?;

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!("✅ Post In Content Feed At Index {} Deleted", index);

    Ok(())
}

async fn get_feed(ipfs: &IpfsClient) -> Result<Feed, Error> {
    let mut res = ipfs.key_list().await?;

    let keypair = match search_keypairs(&FEED_KEY, &mut res) {
        Some(kp) => kp,
        None => return Err(Error::Uncategorized("Key Not Found".into())),
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", &keypair.name, &keypair.id);

    let res = ipfs.name_resolve(Some(&keypair.id), false, false).await?;

    let cid = Cid::try_from(res.path).expect("Invalid Cid");

    ipfs.pin_rm(&cid.to_string(), false).await?;

    let node = ipfs_dag_get_node_async(ipfs, &cid.to_string()).await?;

    Ok(node)
}

async fn get_video_duration(ipfs: &IpfsClient, video: &Cid) -> Result<f64, Error> {
    let path = format!("{}/time", video.to_string());

    let days: DayNode = ipfs_dag_get_node_async(ipfs, &path).await?;

    let mut duration = 0.0;

    for (i, ipld) in days.links_to_hours.iter().enumerate().rev().take(1) {
        duration += (i * 3600) as f64; // 3600 second in 1 hour

        let hours: HourNode = ipfs_dag_get_node_async(ipfs, &ipld.link.to_string()).await?;

        for (i, ipld) in hours.links_to_minutes.iter().enumerate().rev().take(1) {
            duration += (i * 60) as f64; // 60 second in 1 minute

            let minutes: MinuteNode = ipfs_dag_get_node_async(ipfs, &ipld.link.to_string()).await?;

            duration += (minutes.links_to_seconds.len() - 1) as f64;
        }
    }

    Ok(duration)
}
