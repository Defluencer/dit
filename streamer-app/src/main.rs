mod chat;
mod chronicler;
mod config;
mod dag_nodes;
mod ffmpeg_transcoding;
mod server;
mod services;
mod video;

use crate::chat::ChatAggregator;
use crate::chronicler::Chronicler;
use crate::config::Config;
use crate::video::VideoAggregator;

use tokio::sync::mpsc::channel;

use ipfs_api::IpfsClient;

#[tokio::main]
async fn main() {
    println!("Streamer Application Initialization...");

    let ipfs = IpfsClient::default();

    let config = config::get_config(&ipfs).await;

    let (archive_tx, archive_rx) = channel(25);
    let mut chronicler = Chronicler::new(ipfs.clone(), archive_rx).await;

    let (video_tx, video_rx) = channel(config.variants);
    let mut video = VideoAggregator::new(ipfs.clone(), video_rx, archive_tx.clone()).await;

    let mut chat = ChatAggregator::new(ipfs.clone(), archive_tx.clone()).await;

    if config.streamer_app.ffmpeg.is_some() {
        tokio::join!(
            chronicler.collect(),
            chat.aggregate(),
            video.aggregate(),
            server::start_server(video_tx, archive_tx, config.clone()),
            ffmpeg_transcoding::start(config),
        );
    } else {
        tokio::join!(
            chronicler.collect(),
            chat.aggregate(),
            video.aggregate(),
            server::start_server(video_tx, archive_tx, config),
        );
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
