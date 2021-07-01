use crate::utils::dag_nodes::{
    ipfs_dag_get_node_async, ipfs_dag_put_node_async, search_keypairs, update_ipns,
};

use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::blog::FullPost;
use linked_data::feed::Feed;
use linked_data::video::{DayNode, HourNode, MinuteNode, VideoMetadata};
use linked_data::IPLDLink;

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

    let mut feed = get_feed(&ipfs).await?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs();

    let metadata = FullPost {
        title: command.title,
        image: IPLDLink {
            link: command.image,
        },
        content: IPLDLink {
            link: command.content,
        },
        timestamp,
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    feed.content.push(cid.into());

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!(
        "✅ Weblog Post Added In Content Feed At Index {}",
        (feed.content.len() - 1)
    );

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

    let mut feed = get_feed(&ipfs).await?;

    let duration = get_video_duration(&ipfs, command.video).await?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs();

    let metadata = VideoMetadata {
        title: command.title,
        duration,
        image: IPLDLink {
            link: command.image,
        },
        video: IPLDLink {
            link: command.video,
        },
        timestamp,
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    feed.content.push(cid.into());

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!(
        "✅ Video Post Added In Content Feed At Index {}",
        (feed.content.len() - 1)
    );

    Ok(())
}

#[derive(Debug, StructOpt)]
enum UpdateContent {
    /// Create new blog post.
    Blog(UpdatePost),

    /// Create new video.
    Videos(UpdateVideo),
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
    println!("Updating Weblog Post...");
    let ipfs = IpfsClient::default();

    let mut feed = get_feed(&ipfs).await?;

    let cid = match feed.content.get(command.index) {
        Some(mt) => mt.link,
        None => return Err(Error::Uncategorized("Blog Post Index Not Found".into())),
    };

    let mut metadata: FullPost = ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await?;

    if let Some(title) = command.title {
        metadata.title = title;
    }

    if let Some(img) = command.image {
        metadata.image = img.into();
    }

    if let Some(vid) = command.content {
        metadata.content = vid.into();
    }

    metadata.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs();

    let cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    feed.content[command.index] = cid.into();

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!(
        "✅ Weblog Post Updated In Content Feed At Index {}",
        command.index
    );

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
    println!("Updating Video Post...");
    let ipfs = IpfsClient::default();

    let mut feed = get_feed(&ipfs).await?;

    let cid = match feed.content.get(command.index) {
        Some(mt) => mt.link,
        None => return Err(Error::Uncategorized("Video Index Not Found".into())),
    };

    let mut metadata: VideoMetadata = ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await?;

    if let Some(title) = command.title {
        metadata.title = title;
    }

    if let Some(img) = command.image {
        metadata.image = img.into();
    }

    if let Some(vid) = command.video {
        metadata.video = vid.into();
    }

    metadata.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs();

    let cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    feed.content[command.index] = cid.into();

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!(
        "✅ Video Post Updated In Content Feed At Index {}",
        command.index
    );

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

    let mut feed = get_feed(&ipfs).await?;

    feed.content.remove(command.index);

    update_ipns(&ipfs, &FEED_KEY, &feed).await?;

    println!("✅ Post In Content Feed At Index {} Deleted", command.index);

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

    ipfs.pin_rm(&cid.to_string(), true).await?;

    let node = ipfs_dag_get_node_async(ipfs, &cid.to_string()).await?;

    Ok(node)
}

async fn get_video_duration(ipfs: &IpfsClient, video: Cid) -> Result<f64, Error> {
    let path = format!("{}/time", video.to_string());

    let days: DayNode = ipfs_dag_get_node_async(&ipfs, &path).await?;

    let mut duration = 0.0;

    for (i, ipld) in days.links_to_hours.iter().enumerate().rev().take(1) {
        duration += (i * 3600) as f64; // 3600 second in 1 hour

        let hours: HourNode = ipfs_dag_get_node_async(&ipfs, &ipld.link.to_string()).await?;

        for (i, ipld) in hours.links_to_minutes.iter().enumerate().rev().take(1) {
            duration += (i * 60) as f64; // 60 second in 1 minute

            let minutes: MinuteNode =
                ipfs_dag_get_node_async(&ipfs, &ipld.link.to_string()).await?;

            duration += (minutes.links_to_seconds.len() - 1) as f64;
        }
    }

    Ok(duration)
}
