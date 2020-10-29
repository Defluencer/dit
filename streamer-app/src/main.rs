//mod chat;
mod chronicler;
mod config;
mod dag_nodes;
mod ffmpeg_transcoding;
mod server;
mod services;
mod video;

//use crate::chat::ChatAggregator;
use crate::chronicler::Chronicler;
use crate::config::{Config, Ffmpeg, StreamerApp, Topics};
use crate::video::VideoAggregator;

use std::net::SocketAddr;

use tokio::sync::mpsc::channel;

use ipfs_api::IpfsClient;

#[tokio::main]
async fn main() {
    println!("Streamer Application Initialization...");

    let ipfs = IpfsClient::default();

    //let config = config::get_config(&ipfs).await;
    let config = Config {
        gossipsub_topics: Topics {
            video: "livelike".into(),
            chat: "livelikechat".into(),
        },
        streamer_app: StreamerApp {
            socket_addr: "127.0.0.1:2526".into(),
            ffmpeg: Some(Ffmpeg {
                socket_addr: "127.0.0.1:2525".into(),
            }),
        },
        variants: 4,
        video_segment_duration: 4,
    };

    let (archive_tx, archive_rx) = channel(25);
    let mut chronicler = Chronicler::new(ipfs.clone(), archive_rx).await;

    let (video_tx, video_rx) = channel(config.variants);
    let mut video = VideoAggregator::new(ipfs.clone(), video_rx, archive_tx.clone()).await;

    //let mut chat = ChatAggregator::new(ipfs.clone(), archive_tx.clone()).await;

    let streamer_app_addr = config.streamer_app.socket_addr;

    let server_addr = streamer_app_addr
        .parse::<SocketAddr>()
        .expect("Parsing socket address failed");

    if config.streamer_app.ffmpeg.is_some() {
        let ffmpeg_addr = config.streamer_app.ffmpeg.unwrap().socket_addr;

        tokio::join!(
            chronicler.collect(),
            //chat.aggregate(),
            video.aggregate(),
            server::start_server(server_addr, video_tx, archive_tx),
            ffmpeg_transcoding::start(ffmpeg_addr, streamer_app_addr),
        );
    } else {
        tokio::join!(
            chronicler.collect(),
            //chat.aggregate(),
            video.aggregate(),
            server::start_server(server_addr, video_tx, archive_tx),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_config_test() {
        let ipfs = IpfsClient::default();

        let config = crate::config::_get_config(&ipfs).await;

        assert_eq!(config.variants, 4);
    }
}
