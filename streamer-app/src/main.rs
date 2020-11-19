mod actors;
mod config;
mod dag_nodes;
mod server;

use crate::actors::{start_transcoding, Archivist, ChatAggregator, VideoAggregator};
use crate::server::start_server;

use std::net::SocketAddr;

use tokio::sync::mpsc::channel;

use ipfs_api::IpfsClient;

#[tokio::main]
async fn main() {
    println!("Initialization...");

    let ipfs = IpfsClient::default();

    let config = config::get_config(&ipfs).await;

    let (archive_tx, archive_rx) = channel(25);
    let mut archivist = Archivist::new(ipfs.clone(), archive_rx, config.video_segment_duration);

    let (video_tx, video_rx) = channel(config.variants);
    let mut video = VideoAggregator::new(
        ipfs.clone(),
        video_rx,
        archive_tx.clone(),
        config.gossipsub_topics.video,
        config.variants,
    );

    let mut chat = ChatAggregator::new(
        ipfs.clone(),
        archive_tx.clone(),
        config.gossipsub_topics.chat,
    );

    let server_addr = config
        .streamer_app
        .socket_addr
        .parse::<SocketAddr>()
        .expect("Parsing socket address failed");

    match config.streamer_app.ffmpeg {
        Some(ffmpeg) => {
            tokio::join!(
                archivist.collect(),
                chat.aggregate(),
                video.aggregate(),
                start_server(server_addr, video_tx, archive_tx),
                start_transcoding(ffmpeg.socket_addr, config.streamer_app.socket_addr),
            );
        }
        None => {
            tokio::join!(
                archivist.collect(),
                chat.aggregate(),
                video.aggregate(),
                start_server(server_addr, video_tx, archive_tx),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_config_test() {
        let ipfs = IpfsClient::default();

        let config = crate::config::get_config(&ipfs).await;

        assert_eq!(config.variants, 4);
    }
}
