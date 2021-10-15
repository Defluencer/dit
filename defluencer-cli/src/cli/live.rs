use crate::utils::dag_nodes::{get_from_ipns, update_ipns};

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::live::Live;

use structopt::StructOpt;

pub const LIVE_KEY: &str = "live";

#[derive(Debug, StructOpt)]
pub struct LiveCLI {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Choose pubsub topics used for the live stream.
    Topics(UpdateTopics),

    /// Choose the IPFS node that will be streaming.
    PeerID(UpdatePeerId),
}

pub async fn live_cli(cli: LiveCLI) {
    let res = match cli.cmd {
        Command::Topics(topics) => update_topics(topics).await,
        Command::PeerID(peer) => update_peer_id(peer).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {:#?}", e);
    }
}

#[derive(Debug, StructOpt)]
pub struct UpdateTopics {
    /// Pubsub topic for live chat.
    #[structopt(short, long)]
    chat: Option<String>,

    /// Pubsub topic for live video.
    #[structopt(short, long)]
    video: Option<String>,
}

async fn update_topics(command: UpdateTopics) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let UpdateTopics { chat, video } = command;

    let (old_live_cid, mut live) = get_from_ipns::<Live>(&ipfs, LIVE_KEY).await?;

    if let Some(chat_topic) = chat {
        live.chat_topic = chat_topic;
    }

    if let Some(video_topic) = video {
        live.video_topic = video_topic;
    }

    update_ipns(&ipfs, LIVE_KEY, &live).await?;

    let ofc = old_live_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&ofc, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", ofc, e);
    }

    println!("✅ Display Name Updated");

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UpdatePeerId {
    /// Streaming node peer ID.
    #[structopt(short, long)]
    peer_id: String,
}

async fn update_peer_id(command: UpdatePeerId) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let UpdatePeerId { peer_id } = command;

    let (old_live_cid, mut live) = get_from_ipns::<Live>(&ipfs, LIVE_KEY).await?;

    live.peer_id = peer_id;

    update_ipns(&ipfs, LIVE_KEY, &live).await?;

    let ofc = old_live_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&ofc, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", ofc, e);
    }

    println!("✅ Avatar Updated");

    Ok(())
}
