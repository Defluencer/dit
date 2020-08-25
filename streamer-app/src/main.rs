mod collector;
mod config;
mod ffmpeg_transcoding;
mod hash_timecode;
mod server;
mod services;

use crate::collector::HashVideo;
use crate::config::Config;
use crate::hash_timecode::HashTimecode;

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

    match ipfs.config_get_string("Identity.PeerID").await {
        Ok(peer_id) => {
            if peer_id.value == config.streamer_peer_id {
                println!("Peer Id: {}", peer_id.value);
            } else {
                eprintln!("Error! {} != {}", peer_id.value, config.streamer_peer_id);
                return;
            }
        }
        Err(_) => {
            eprintln!("Error! Is IPFS running with PubSub enabled?");
            return;
        }
    }

    let (timecode_tx, timecode_rx) = channel(5);

    let mut timecode = HashTimecode::new(ipfs.clone(), timecode_rx, config.clone());

    let (video_tx, video_rx) = channel(config.variants);

    let mut video = HashVideo::new(ipfs.clone(), video_rx, timecode_tx.clone(), config.clone());

    if config.streamer_app.ffmpeg.is_some() {
        tokio::join!(
            video.collect(),
            server::start_server(video_tx, timecode_tx, config.clone()),
            timecode.collect(),
            ffmpeg_transcoding::start(config)
        );
    } else {
        tokio::join!(
            video.collect(),
            server::start_server(video_tx, timecode_tx, config),
            timecode.collect()
        );
    }
}
