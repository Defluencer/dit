use crate::utils::dag_nodes::{
    get_from_ipns, ipfs_dag_get_node_async, ipfs_dag_put_node_async, update_ipns,
};

use serde::de::DeserializeOwned;
use serde::Serialize;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::comments::{Comments, CommentsAnchor};
use linked_data::feed::FeedAnchor;
use linked_data::video::{DayNode, HourNode, MinuteNode, VideoMetadata};

use cid::Cid;

use structopt::StructOpt;

pub const FEED_KEY: &str = "feed";
pub const COMMENTS_KEY: &str = "comments";

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
        eprintln!("❗ IPFS: {:#?}", e);
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

    let index = add_content_to_feed(&ipfs, &metadata).await?;

    println!("✅ Weblog Added At Index {}", index);

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

    let index = add_content_to_feed(&ipfs, &metadata).await?;

    println!("✅ Weblog Added At Index {}", index);

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

    let index = add_content_to_feed(&ipfs, &metadata).await?;

    println!("✅ Video Added At Index {}", index);

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

    let UpdateMicroPost { index, content } = command;

    let (mut feed, mut metadata) = unload_feed::<MicroPost>(&ipfs, index).await?;

    metadata.update(content);

    tokio::try_join!(
        reload_feed(&ipfs, index, &metadata, &mut feed),
        reset_comments_at(&ipfs, index)
    )?;

    println!("✅ Weblog Updated & Comments Cleared At Index {}", index);

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

    let UpdatePost {
        index,
        title,
        image,
        content,
    } = command;

    let (mut feed, mut metadata) = unload_feed::<FullPost>(&ipfs, index).await?;

    metadata.update(title, image, content);

    tokio::try_join!(
        reload_feed(&ipfs, index, &metadata, &mut feed),
        reset_comments_at(&ipfs, index)
    )?;

    println!("✅ Weblog Updated & Comments Cleared At Index {}", index);

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

    let UpdateVideo {
        index,
        title,
        image,
        video,
    } = command;

    let (mut feed, mut metadata) = unload_feed::<VideoMetadata>(&ipfs, index).await?;

    let duration = match video {
        Some(cid) => Some(get_video_duration(&ipfs, &cid).await?),
        None => None,
    };

    metadata.update(title, image, video, duration);

    tokio::try_join!(
        reload_feed(&ipfs, index, &metadata, &mut feed),
        reset_comments_at(&ipfs, index)
    )?;

    println!("✅ Video Updated & Comments Cleared At Index {}", index);

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

    let (mut feed, mut comments) = tokio::try_join!(
        get_from_ipns::<FeedAnchor>(&ipfs, FEED_KEY),
        get_from_ipns::<CommentsAnchor>(&ipfs, COMMENTS_KEY)
    )?;

    if index >= feed.content.len() || index >= comments.links.len() {
        return Err(Error::Uncategorized("Index Not Found".into()));
    }

    let content = feed.content.remove(index);
    let comments = comments.links.remove(index);

    let content_cid = content.link.to_string();
    let comment_cid = comments.link.to_string();

    tokio::try_join!(
        ipfs.pin_rm(&content_cid, true),
        ipfs.pin_rm(&comment_cid, true),
        update_ipns(&ipfs, FEED_KEY, &feed),
        update_ipns(&ipfs, COMMENTS_KEY, &comments)
    )?;

    println!("✅ Post & Comments Deleted At Index {}", index);

    Ok(())
}

/*** Utils below ****/

/// Serialize and pin content then update IPNS.
async fn add_content_to_feed<T>(ipfs: &IpfsClient, metadata: &T) -> Result<usize, Error>
where
    T: Serialize,
{
    let comments = Comments::default();

    let (content_cid, comments_cid) = tokio::try_join!(
        ipfs_dag_put_node_async(ipfs, metadata),
        ipfs_dag_put_node_async(ipfs, &comments)
    )?;

    println!("New Content => {:?}", &content_cid);

    let content_cid_string = content_cid.to_string();
    let comment_cid_string = comments_cid.to_string();

    println!("Pinning...");

    tokio::try_join!(
        ipfs.pin_add(&content_cid_string, true),
        ipfs.pin_add(&comment_cid_string, true)
    )?;

    println!("Updating Content Feed...");

    let (mut feed, mut comments) = tokio::try_join!(
        get_from_ipns::<FeedAnchor>(ipfs, FEED_KEY),
        get_from_ipns::<CommentsAnchor>(ipfs, COMMENTS_KEY)
    )?;

    feed.content.push(content_cid.into());
    comments.links.push(comments_cid.into());

    tokio::try_join!(
        update_ipns(ipfs, FEED_KEY, &feed),
        update_ipns(ipfs, COMMENTS_KEY, &comments)
    )?;

    Ok(feed.content.len() - 1)
}

/// Get comment anchor, update with empty comment list then update IPNS.
async fn reset_comments_at(ipfs: &IpfsClient, index: usize) -> Result<(), Error> {
    println!("Clearing Comments...");

    let mut comments = get_from_ipns::<CommentsAnchor>(ipfs, COMMENTS_KEY).await?;

    let new_cid = ipfs_dag_put_node_async(ipfs, &Comments::default()).await?;
    comments.links[index] = new_cid.into();

    update_ipns(ipfs, COMMENTS_KEY, &comments).await?;

    Ok(())
}

/// Get cid at index in feed, unpin then return feed and cid.
async fn unload_feed<T>(ipfs: &IpfsClient, index: usize) -> Result<(FeedAnchor, T), Error>
where
    T: DeserializeOwned,
{
    let feed = get_from_ipns::<FeedAnchor>(ipfs, FEED_KEY).await?;

    let old_cid = match feed.content.get(index) {
        Some(mt) => mt.link,
        None => return Err(Error::Uncategorized("Index Not Found".into())),
    };
    println!("Old Post => {:?}", old_cid);

    println!("Unpinning...");
    ipfs.pin_rm(&old_cid.to_string(), true).await?;

    let metadata: T = ipfs_dag_get_node_async(ipfs, &old_cid.to_string()).await?;

    Ok((feed, metadata))
}

/// Serialize and pin metadata then update feed and update IPNS.
async fn reload_feed<T>(
    ipfs: &IpfsClient,
    index: usize,
    metadata: &T,
    feed: &mut FeedAnchor,
) -> Result<(), Error>
where
    T: Serialize,
{
    let new_cid = ipfs_dag_put_node_async(ipfs, metadata).await?;
    println!("New Post => {:?}", new_cid);

    println!("Pinning...");
    ipfs.pin_add(&new_cid.to_string(), true).await?;

    println!("Updating Content Feed...");
    feed.content[index] = new_cid.into();
    update_ipns(ipfs, FEED_KEY, feed).await?;

    Ok(())
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
