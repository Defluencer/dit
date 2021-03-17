use crate::beacon::search_keypairs;
use crate::utils::dag_nodes::{ipfs_dag_get_node_async, ipfs_dag_put_node_async};
use crate::DEFAULT_KEY;

use std::convert::TryFrom;

use ipfs_api::IpfsClient;

use linked_data::beacon::VideoList;
use linked_data::video::VideoMetadata;
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

    /// The new video duration.
    /// Must be less than actual video duration, -0.1s is fine.
    /// Should be fixed in future versions.
    #[structopt(short, long)]
    duration: f64,

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

    /// The new video duration.
    /// Must be less than actual video duration, -0.1s is fine.
    /// Should be fixed in future versions.
    #[structopt(short, long)]
    duration: Option<f64>,

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
    match cli.cmd {
        Command::Add(add) => add_video(add, cli.key_name).await,
        Command::Update(update) => update_video(update, cli.key_name).await,
        Command::Delete(delete) => delete_video(delete, cli.key_name).await,
    }
}

async fn add_video(command: Add, key: String) {
    let ipfs = IpfsClient::default();

    let mut video_list = match get_video_list(&ipfs, &key).await {
        Some(vl) => vl,
        None => return,
    };

    let metadata = VideoMetadata {
        title: command.title,
        duration: command.duration,
        image: IPLDLink {
            link: command.image,
        },
        video: IPLDLink {
            link: command.video,
        },
    };

    let cid = match ipfs_dag_put_node_async(&ipfs, &metadata).await {
        Ok(cid) => cid,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return;
        }
    };

    video_list.metadata.push(IPLDLink { link: cid });

    update_video_list(&ipfs, &key, &video_list).await;
}

async fn update_video(command: Update, key: String) {
    let ipfs = IpfsClient::default();

    let mut video_list = match get_video_list(&ipfs, &key).await {
        Some(vl) => vl,
        None => return,
    };

    let cid = match video_list.metadata.get(command.index) {
        Some(mt) => mt.link,
        None => {
            eprintln!("Video not found");
            return;
        }
    };

    let mut metadata: VideoMetadata = match ipfs_dag_get_node_async(&ipfs, &cid.to_string()).await {
        Ok(node) => node,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return;
        }
    };

    if let Some(titl) = command.title {
        metadata.title = titl;
    }

    if let Some(dur) = command.duration {
        metadata.duration = dur;
    }

    if let Some(img) = command.image {
        metadata.image = IPLDLink { link: img };
    }

    if let Some(vid) = command.video {
        metadata.video = IPLDLink { link: vid };
    }

    let cid = match ipfs_dag_put_node_async(&ipfs, &metadata).await {
        Ok(cid) => cid,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return;
        }
    };

    video_list.metadata[command.index] = IPLDLink { link: cid };

    update_video_list(&ipfs, &key, &video_list).await;
}

async fn delete_video(command: Delete, key: String) {
    let ipfs = IpfsClient::default();

    let mut video_list = match get_video_list(&ipfs, &key).await {
        Some(vl) => vl,
        None => return,
    };

    video_list.metadata.remove(command.index);

    update_video_list(&ipfs, &key, &video_list).await;
}

/// Get video list associated with IPNS key, unpin it then return it.
async fn get_video_list(ipfs: &IpfsClient, key: &str) -> Option<VideoList> {
    let res = match ipfs.key_list().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return None;
        }
    };

    let keypair = match search_keypairs(key, res) {
        Some(kp) => kp,
        None => {
            eprintln!("Key not found");
            return None;
        }
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", &keypair.name, &keypair.id);

    let cid = match ipfs.name_resolve(Some(&keypair.id), false, false).await {
        Ok(res) => Cid::try_from(res.path).expect("Invalid Cid"),
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return None;
        }
    };

    if let Err(e) = ipfs.pin_rm(&cid.to_string(), false).await {
        eprintln!("IPFS: {}", e);
        return None;
    }

    match ipfs_dag_get_node_async(ipfs, &cid.to_string()).await {
        Ok(node) => Some(node),
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return None;
        }
    }
}

/// Serialize the new video list, pin it then publish it under this IPNS key.
pub async fn update_video_list(ipfs: &IpfsClient, key: &str, video_list: &VideoList) {
    let cid = match ipfs_dag_put_node_async(ipfs, video_list).await {
        Ok(cid) => cid,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return;
        }
    };

    if let Err(e) = ipfs.pin_add(&cid.to_string(), true).await {
        eprintln!("IPFS: {}", e);
        return;
    }

    println!("Updating Video List...");

    if let Err(e) = ipfs
        .name_publish(&cid.to_string(), false, None, None, Some(key))
        .await
    {
        eprintln!("IPFS: {}", e);
        return;
    }

    println!("Video List CID => {}", &cid.to_string());
}
