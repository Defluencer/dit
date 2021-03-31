use crate::beacon::search_keypairs;
use crate::utils::dag_nodes::{ipfs_dag_get_node_async, ipfs_dag_put_node_async};
use crate::DEFAULT_KEY;

use std::convert::TryFrom;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::beacon::VideoList;
use linked_data::video::{DayNode, HourNode, MinuteNode, VideoMetadata};
use linked_data::IPLDLink;

use cid::Cid;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Video {
    /// IPNS key name for video list resolution.
    #[structopt(short, long, default_value = DEFAULT_KEY)]
    key_name: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Create a new video metadata.
    Add(Add),

    /// Update video metadata in video list.
    Update(Update),

    /// Delete video metadata in video list.
    Delete(Delete),
}

#[derive(Debug, StructOpt)]
pub struct Add {
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

#[derive(Debug, StructOpt)]
pub struct Update {
    /// The index of the video to update.
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

#[derive(Debug, StructOpt)]
pub struct Delete {
    /// The index of the video to delete.
    #[structopt(short, long)]
    index: usize,
}

pub async fn video_cli(cli: Video) {
    let res = match cli.cmd {
        Command::Add(add) => add_video(add, cli.key_name).await,
        Command::Update(update) => update_video(update, cli.key_name).await,
        Command::Delete(delete) => delete_video(delete, cli.key_name).await,
    };

    if let Err(e) = res {
        eprintln!("IPFS: {}", e);
    }
}

async fn add_video(command: Add, key: String) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let mut video_list = get_video_list(&ipfs, &key).await?;

    let duration = get_video_duration(&ipfs, command.video).await?;

    let metadata = VideoMetadata {
        title: command.title,
        duration,
        image: IPLDLink {
            link: command.image,
        },
        video: IPLDLink {
            link: command.video,
        },
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    video_list.metadata.push(IPLDLink { link: cid });

    update_video_list(&ipfs, &key, &video_list).await?;

    Ok(())
}

async fn update_video(command: Update, key: String) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let mut video_list = get_video_list(&ipfs, &key).await?;

    let cid = match video_list.metadata.get(command.index) {
        Some(mt) => mt.link,
        None => return Err(Error::Uncategorized("Video Index Not Found".into())),
    };

    let mut metadata: VideoMetadata = ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await?;

    if let Some(titl) = command.title {
        metadata.title = titl;
    }

    if let Some(img) = command.image {
        metadata.image = IPLDLink { link: img };
    }

    if let Some(vid) = command.video {
        metadata.video = IPLDLink { link: vid };
    }

    let cid = ipfs_dag_put_node_async(&ipfs, &metadata).await?;

    video_list.metadata[command.index] = IPLDLink { link: cid };

    update_video_list(&ipfs, &key, &video_list).await?;

    Ok(())
}

async fn delete_video(command: Delete, key: String) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let mut video_list = get_video_list(&ipfs, &key).await?;

    let _cid = video_list.metadata.remove(command.index).link;

    update_video_list(&ipfs, &key, &video_list).await?;

    Ok(())
}

/// Get video list associated with IPNS key, unpin it then return it.
async fn get_video_list(ipfs: &IpfsClient, key: &str) -> Result<VideoList, Error> {
    let res = ipfs.key_list().await?;

    let keypair = match search_keypairs(key, res) {
        Some(kp) => kp,
        None => return Err(Error::Uncategorized("Key Not Found".into())),
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", &keypair.name, &keypair.id);

    println!("Fetching Video List...");

    let res = ipfs.name_resolve(Some(&keypair.id), false, false).await?;

    let cid = Cid::try_from(res.path).expect("Invalid Cid");

    ipfs.pin_rm(&cid.to_string(), true).await?;

    let node = ipfs_dag_get_node_async(ipfs, &cid.to_string()).await?;

    Ok(node)
}

/// Serialize the new video list, pin it then publish it under this IPNS key.
pub async fn update_video_list(
    ipfs: &IpfsClient,
    key: &str,
    video_list: &VideoList,
) -> Result<(), Error> {
    let cid = ipfs_dag_put_node_async(ipfs, video_list).await?;

    ipfs.pin_add(&cid.to_string(), true).await?;

    println!("Updating Video List...");

    ipfs.name_publish(&cid.to_string(), false, None, None, Some(key))
        .await?;

    println!("Video List CID => {}", &cid.to_string());

    Ok(())
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

            for (i, _) in minutes.links_to_seconds.iter().enumerate().rev().take(1) {
                duration += i as f64;
            }
        }
    }

    Ok(duration)
}
