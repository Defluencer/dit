use tokio::stream::StreamExt;

use hyper::body::Bytes;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::{Deserialize, Serialize};

pub async fn get_config(ipfs: &IpfsClient) -> Config {
    let config = ipfs
        .name_resolve(None, false, false)
        .await
        .expect("IPFS name resolve failed");

    let buffer: Result<Bytes, Error> = ipfs.dag_get(&config.path).collect().await;

    let buffer = buffer.expect("IPFS DAG get failed");

    serde_json::from_slice(&buffer).expect("Deserializing config failed")
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Config {
    pub streamer_peer_id: String,
    pub gossipsub_topics: Topics,
    pub streamer_app: StreamerApp,
    pub variants: usize,
    pub video_segment_duration: usize,
    pub pin_stream: bool,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct StreamerApp {
    pub socket_addr: String,
    pub ffmpeg: Option<Ffmpeg>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Ffmpeg {
    pub socket_addr: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Topics {
    pub video: String,
    pub chat: String,
}

//TODO impl default
