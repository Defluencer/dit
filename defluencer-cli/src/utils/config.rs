use tokio::fs;

use std::io::Error;
use std::net::SocketAddr;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

const CONFIG_LOCATION: &str = "config.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct ArchiveConfig {
    #[serde(skip)]
    pub archive_live_chat: bool, // get from argument not file
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoConfig {
    #[serde(skip)]
    pub pubsub_enable: bool, // get from argument not file

    pub pubsub_topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatConfig {
    pub topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub input_socket_addr: SocketAddr,
    pub archive: ArchiveConfig,
    pub video: VideoConfig,
    pub chat: ChatConfig,
}

impl Configuration {
    pub async fn from_file() -> Result<Self, Error> {
        let config = fs::read(CONFIG_LOCATION).await?;
        let config = serde_json::from_slice::<Self>(&config)?;

        Ok(config)
    }

    pub async fn save_to_file(&self) -> Result<(), Error> {
        let data = serde_json::to_vec_pretty(&self)?;

        fs::write(CONFIG_LOCATION, data).await
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            input_socket_addr: SocketAddr::from_str("127.0.0.1:2526").expect("Invalid Address"),

            archive: ArchiveConfig {
                archive_live_chat: true,
            },

            video: VideoConfig {
                pubsub_enable: true,
                pubsub_topic: "defluencer_live_video".into(),
            },

            chat: ChatConfig {
                topic: "defluencer_live_chat".into(),
            },
        }
    }
}
