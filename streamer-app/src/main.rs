mod actors;
mod server;
mod utils;

use crate::actors::{Archivist, ChatAggregator, VideoAggregator};
use crate::server::start_server;
use crate::utils::get_config;

use tokio::sync::mpsc::unbounded_channel;

use ipfs_api::IpfsClient;

use linked_data::config::Configuration;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// Disable all archiving. Only affect live streaming.
    #[structopt(long)]
    no_archive: bool,

    /// Disable chat archiving fonctionalities. Recommended when not streaming.
    #[structopt(long)]
    no_chat: bool,

    /// Disable publish subscribe channel. Recommended when not streaming.
    #[structopt(long)]
    no_pubsub: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    if opt.no_archive && opt.no_pubsub {
        eprintln!("Cannot Disable Archiving And PubSub! Aborting...");
        return;
    }

    let ipfs = IpfsClient::default();

    match ipfs.id(None).await {
        Ok(res) => println!("IPFS: Peer ID => {}", res.id),
        Err(_) => {
            eprintln!("IPFS must be started beforehand. Aborting...");
            return;
        }
    }

    println!("Initialization...");

    let config = get_config().await;

    let Configuration {
        input_socket_addr,
        mut archive,
        mut video,
        chat,
    } = config;

    let mut handles = Vec::with_capacity(4);

    let topic = chat.pubsub_topic.clone();

    let archive_tx = {
        if !opt.no_archive {
            let (archive_tx, archive_rx) = unbounded_channel();

            if !opt.no_chat {
                let mut chat = ChatAggregator::new(ipfs.clone(), archive_tx.clone(), chat);

                let chat_handle = tokio::spawn(async move {
                    chat.start().await;
                });

                handles.push(chat_handle);
            }

            archive.archive_live_chat = !opt.no_chat;

            let mut archivist = Archivist::new(ipfs.clone(), archive_rx, archive);

            let archive_handle = tokio::spawn(async move {
                archivist.start().await;
            });

            handles.push(archive_handle);

            Some(archive_tx)
        } else {
            None
        }
    };

    let (video_tx, video_rx) = unbounded_channel();

    video.pubsub_enable = !opt.no_pubsub;

    let mut video = VideoAggregator::new(ipfs.clone(), video_rx, archive_tx.clone(), video);

    let video_handle = tokio::spawn(async move {
        video.start().await;
    });

    handles.push(video_handle);

    let server_handle = tokio::spawn(async move {
        start_server(input_socket_addr, video_tx, archive_tx, ipfs, topic).await;
    });

    handles.push(server_handle);

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Main: {}", e);
        }
    }
}
