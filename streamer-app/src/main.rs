mod collector;
mod ffmpeg_transcoding;
mod server;
mod services;

use std::fs::File;
use std::io::BufReader;

use tokio::sync::mpsc::channel;

use serde::Deserialize;

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

    let (tx, rx) = channel(4);

    if config.streamer_app.ffmpeg.is_some() {
        tokio::join!(
            collector::collect_video_data(ipfs, rx, &config),
            server::start_server(tx, &config),
            ffmpeg_transcoding::start(&config)
        );
    } else {
        tokio::join!(
            collector::collect_video_data(ipfs, rx, &config),
            server::start_server(tx, &config)
        );
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub streamer_peer_id: String,
    pub gossipsub_topic: String,
    pub streamer_app: StreamerApp,
}

#[derive(Debug, Deserialize)]
pub struct StreamerApp {
    pub socket_addr: String,
    pub ffmpeg: Option<Ffmpeg>,
}

#[derive(Debug, Deserialize)]
pub struct Ffmpeg {
    pub socket_addr: String,
}
