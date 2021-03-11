mod actors;
mod server;
mod utils;

use crate::actors::{start_transcoding, Archivist, ChatAggregator, VideoAggregator};
use crate::server::start_server;
use crate::utils::get_config;

use tokio::sync::mpsc::unbounded_channel;

use ipfs_api::IpfsClient;

#[tokio::main]
async fn main() {
    println!("Initialization...");

    let ipfs = IpfsClient::default();

    let config = get_config().await;

    let mut handles = Vec::with_capacity(4);

    let archive_tx = {
        if let Some(archive_config) = config.archive {
            let (archive_tx, archive_rx) = unbounded_channel();

            if archive_config.archive_live_chat {
                let mut chat = ChatAggregator::new(ipfs.clone(), archive_tx.clone(), config.chat);

                let chat_handle = tokio::spawn(async move {
                    chat.start_receiving().await;
                });

                handles.push(chat_handle);
            }

            let mut archivist = Archivist::new(ipfs.clone(), archive_rx, archive_config);

            let archive_handle = tokio::spawn(async move {
                archivist.collect().await;
            });

            handles.push(archive_handle);

            Some(archive_tx)
        } else {
            None
        }
    };

    let (video_tx, video_rx) = unbounded_channel();

    let mut video = VideoAggregator::new(ipfs.clone(), video_rx, archive_tx.clone(), config.video);

    let video_handle = tokio::spawn(async move {
        video.start_receiving().await;
    });

    handles.push(video_handle);

    let server_addr = config.addresses.app_addr.clone();
    let server_addr_clone = config.addresses.app_addr.clone();

    let server_handle = tokio::spawn(async move {
        start_server(server_addr, video_tx, archive_tx, ipfs).await;
    });

    handles.push(server_handle);

    if let Some(ffmpeg_addr) = config.addresses.ffmpeg_addr {
        let ffmpeg_handle = tokio::spawn(async move {
            start_transcoding(ffmpeg_addr, server_addr_clone).await;
        });

        handles.push(ffmpeg_handle);
    }

    //join_all(handles).await;

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Main: {}", e);
        }
    }
}
