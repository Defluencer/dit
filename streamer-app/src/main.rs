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

use std::fs::File;
use std::io::BufReader;

use tokio::sync::mpsc::channel;

use ipfs_api::IpfsClient;

#[tokio::main]
async fn main() {
    println!("Streamer Application Initialization...");

    let file = File::open("livelike_config.json").expect("Opening configuration file failed");
    let reader = BufReader::new(file);

    let config: Config =
        serde_json::from_reader(reader).expect("Deserializing configuration file failed");

    let ipfs = IpfsClient::default();

    let peer_id = match ipfs.config_get_string("Identity.PeerID").await {
        Ok(peer_id) => {
            if peer_id.value == config.streamer_peer_id {
                peer_id.value
            } else {
                eprintln!("Error! {} != {}", peer_id.value, config.streamer_peer_id);
                return;
            }
        }
        Err(_) => {
            eprintln!("Error! Is IPFS running with PubSub enabled?");
            return;
        }
    };

    println!("Peer Id: {}", peer_id);

    let (archive_tx, archive_rx) = channel(25);
    let mut chronicler = Chronicler::new(ipfs.clone(), archive_rx, config.clone());

    let (video_tx, video_rx) = channel(config.variants);
    let mut video =
        VideoAggregator::new(ipfs.clone(), video_rx, archive_tx.clone(), config.clone());

    let mut chat = ChatAggregator::new(ipfs.clone(), archive_tx.clone(), config.clone());

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
