use crate::utils::dag_nodes::{ipfs_dag_get_node_async, ipfs_dag_put_node_async, search_keypairs};

use std::convert::TryFrom;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::blog::FullPost;
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
            AddContent::Blog(blog) => add_blog(blog).await,
            AddContent::Videos(video) => add_video(video).await,
        },
        Command::Update(update) => match update {
            UpdateContent::Blog(blog) => update_blog(blog).await,
            UpdateContent::Videos(video) => update_video(video).await,
        },
        Command::Delete(delete) => delete_content(delete).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {}", e);
    }
}

#[derive(Debug, StructOpt)]
enum AddContent {
    /// Create new blog post.
    Blog(AddPost),

    /// Create new video post.
    Videos(AddVideo),
}

async fn add_content_to_feed(ipfs: &IpfsClient, new_cid: Cid) -> Result<(), Error> {
    ipfs.pin_add(&new_cid.to_string(), true).await?;

    let old_feed_cid = get_feed(ipfs).await?;
    let new_feed = Feed::add(new_cid, old_feed_cid);
    let new_feed_cid = ipfs_dag_put_node_async(ipfs, &new_feed).await?.to_string();
    ipfs.pin_add(&new_feed_cid, false).await?; // feed is not pinned recursively, only content metadata
    ipfs.name_publish(&new_feed_cid, false, None, None, Some(FEED_KEY))
        .await?;

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
    println!("Adding Weblog Post...");

    let ipfs = IpfsClient::default();

    let AddPost {
        title,
        image,
        content,
    } = command;

    let metadata = FullPost::create(title, image, content);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    add_content_to_feed(&ipfs, new_cid).await?;

    println!("✅ Weblog Post Added");

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
    println!("Adding Video Post...");

    let ipfs = IpfsClient::default();

    let AddVideo {
        title,
        image,
        video,
    } = command;

    let duration = get_video_duration(&ipfs, &video).await?;

    let metadata = VideoMetadata::create(title, duration, image, video);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    add_content_to_feed(&ipfs, new_cid).await?;

    println!("✅ Video Post Added");

    Ok(())
}

#[derive(Debug, StructOpt)]
enum UpdateContent {
    /// Create new blog post.
    Blog(UpdatePost),

    /// Create new video.
    Videos(UpdateVideo),
}

async fn update_content_feed(ipfs: &IpfsClient, old_cid: Cid, new_cid: Cid) -> Result<(), Error> {
    ipfs.pin_add(&new_cid.to_string(), true).await?;

    ipfs.pin_rm(&old_cid.to_string(), true).await?;

    let old_feed_cid = get_feed(ipfs).await?;
    let new_feed = Feed::update(new_cid, old_cid, old_feed_cid);
    let new_feed_cid = ipfs_dag_put_node_async(ipfs, &new_feed).await?.to_string();
    ipfs.pin_add(&new_feed_cid, false).await?; // feed is not pinned recursively, only content metadata
    ipfs.name_publish(&new_feed_cid, false, None, None, Some(FEED_KEY))
        .await?;

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UpdatePost {
    /// The content identifier of the post metadata to update.
    #[structopt(long)]
    cid: Cid,

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
    println!("Updating Weblog Post...");

    let ipfs = IpfsClient::default();

    let UpdatePost {
        cid,
        title,
        image,
        content,
    } = command;

    let mut metadata: FullPost = ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await?;

    metadata.update(title, image, content);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    update_content_feed(&ipfs, cid, new_cid).await?;

    println!("✅ Weblog Post Updated");

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UpdateVideo {
    /// The content identifier of the video metadata to update.
    #[structopt(long)]
    cid: Cid,

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
    println!("Updating Video Post...");

    let ipfs = IpfsClient::default();

    let UpdateVideo {
        cid,
        title,
        image,
        video,
    } = command;

    let mut metadata: VideoMetadata = ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await?;

    let duration = match video {
        Some(cid) => Some(get_video_duration(&ipfs, &cid).await?),
        None => None,
    };

    metadata.update(title, image, video, duration);

    let new_cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    update_content_feed(&ipfs, cid, new_cid).await?;

    println!("✅ Video Post Updated");

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct DeleteContent {
    /// The identifier of the content to delete.
    #[structopt(short, long)]
    cid: Cid,
}

async fn delete_content(command: DeleteContent) -> Result<(), Error> {
    println!("Deleting Content...");

    let ipfs = IpfsClient::default();

    ipfs.pin_rm(&command.cid.to_string(), true).await?;

    let old_feed_cid = get_feed(&ipfs).await?;
    let new_feed = Feed::delete(command.cid, old_feed_cid);
    let new_feed_cid = ipfs_dag_put_node_async(&ipfs, &new_feed).await?.to_string();
    ipfs.pin_add(&new_feed_cid, false).await?; // feed is not pinned recursively, only content metadata
    ipfs.name_publish(&new_feed_cid, false, None, None, Some(FEED_KEY))
        .await?;

    println!("✅ Content Deleted");

    Ok(())
}

async fn get_feed(ipfs: &IpfsClient) -> Result<Cid, Error> {
    let mut res = ipfs.key_list().await?;

    let keypair = match search_keypairs(&FEED_KEY, &mut res) {
        Some(kp) => kp,
        None => return Err(Error::Uncategorized("Key Not Found".into())),
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", &keypair.name, &keypair.id);

    let res = ipfs.name_resolve(Some(&keypair.id), false, false).await?;

    let cid = Cid::try_from(res.path).expect("Invalid Cid");

    Ok(cid)
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
