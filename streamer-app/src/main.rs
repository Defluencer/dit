mod collector;
mod config;
mod ffmpeg_transcoding;
mod hash_timecode;
mod server;
mod services;
mod stream_links;

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

    match ipfs.config("Identity.PeerID", None, None, None).await {
        Ok(peer_id) => {
            println!("IPFS PeerId: {}", peer_id.value);
        }
        Err(_) => {
            eprintln!("Error! Is IPFS running with PubSub enabled?");
            return;
        }
    }

    let timecode = HashTimecode::new(ipfs.clone());

    let (tx, rx) = channel(4);

    if config.streamer_app.ffmpeg.is_some() {
        tokio::join!(
            collector::collect_video_data(ipfs, timecode, rx, &config),
            server::start_server(tx, &config),
            ffmpeg_transcoding::start(&config)
        );
    } else {
        tokio::join!(
            collector::collect_video_data(ipfs, timecode, rx, &config),
            server::start_server(tx, &config)
        );
    }
}
