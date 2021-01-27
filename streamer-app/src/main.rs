#![allow(unused_must_use)]

mod actors;
mod server;
mod utils;

use crate::actors::{start_transcoding, Archivist, ChatAggregator, VideoAggregator};
use crate::server::start_server;
use crate::utils::get_config;

use tokio::sync::mpsc::channel;

use ipfs_api::IpfsClient;

#[tokio::main]
async fn main() {
    println!("Initialization...");

    let ipfs = IpfsClient::default();

    let config = get_config().await;

    let (archive_tx, archive_rx) = channel(25);
    let mut archivist = Archivist::new(ipfs.clone(), archive_rx, config.segment_duration);
    let archive_handle = tokio::spawn(async move {
        archivist.collect().await;
    });

    let (video_tx, video_rx) = channel(config.tracks.len());
    let mut video = VideoAggregator::new(
        ipfs.clone(),
        video_rx,
        archive_tx.clone(),
        config.gossipsub_topics.video,
        config.tracks,
    );
    let video_handle = tokio::spawn(async move {
        video.start_receiving().await;
    });

    let mut chat = ChatAggregator::new(
        ipfs.clone(),
        archive_tx.clone(),
        config.gossipsub_topics.chat,
    );
    let chat_handle = tokio::spawn(async move {
        chat.start_receiving().await;
    });

    let server_addr = config.addresses.app_addr.clone();
    let server_addr_clone = config.addresses.app_addr.clone();

    let server_handle = tokio::spawn(async move {
        start_server(server_addr, video_tx, archive_tx).await;
    });

    match config.addresses.ffmpeg_addr {
        Some(ffmpeg_addr) => {
            let ffmpeg_handle = tokio::spawn(async move {
                start_transcoding(ffmpeg_addr, server_addr_clone).await;
            });

            tokio::join!(
                archive_handle,
                chat_handle,
                video_handle,
                ffmpeg_handle,
                server_handle
            );
        }
        None => {
            tokio::join!(archive_handle, chat_handle, video_handle, server_handle);
        }
    }
}
