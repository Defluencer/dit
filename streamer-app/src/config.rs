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

impl Default for Config {
    fn default() -> Self {
        Self {
            gossipsub_topics: Topics {
                video: "livelikevideo".into(),
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
        }
    }
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

pub async fn get_config(ipfs: &IpfsClient) -> Config {
    if let Ok(config) = ipfs.name_resolve(None, false, false).await {
        let buffer: Result<Bytes, Error> = ipfs.dag_get(&config.path).collect().await;

        if let Ok(buffer) = buffer {
            if let Ok(config) = serde_json::from_slice(&buffer) {
                return config;
            }
        }
    }

    let config = Config::default();

    eprintln!("Cannot load config. Fallback -> {:#?}", config);

    config
}
