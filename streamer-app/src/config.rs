//use crate::dag_nodes::IPLDLink;

use tokio::stream::StreamExt;

use hyper::body::Bytes;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub gossipsub_topics: Topics,
    pub streamer_app: StreamerApp,
    pub variants: usize,
    pub video_segment_duration: usize,
    //pub blacklist: IPLDLink,
    //pub whitelist: IPLDLink,
    //pub mods: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Topics {
    pub video: String,
    pub chat: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamerApp {
    pub socket_addr: String,
    pub ffmpeg: Option<Ffmpeg>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ffmpeg {
    pub socket_addr: String,
}

pub async fn _get_config(ipfs: &IpfsClient) -> Config {
    let config = ipfs
        .name_resolve(None, false, false)
        .await
        .expect("IPFS name resolve failed");

    let buffer: Result<Bytes, Error> = ipfs.dag_get(&config.path).collect().await;

    let buffer = buffer.expect("IPFS DAG get failed");

    serde_json::from_slice(&buffer).expect("Deserializing config failed")
}
