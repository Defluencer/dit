use std::net::SocketAddr;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub input_socket_addr: SocketAddr,
    pub archive: ArchiveConfig,
    pub video: VideoConfig,
    pub chat: ChatConfig,
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
                pubsub_topic: "defluencer_live_chat".into(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArchiveConfig {
    #[serde(skip)]
    pub archive_live_chat: bool, // get from argument not file

                                 //pub segment_duration: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoConfig {
    #[serde(skip)]
    pub pubsub_enable: bool, // get from argument not file

    pub pubsub_topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatConfig {
    pub pubsub_topic: String,
    //pub blacklist: IPLDLink,
    //pub whitelist: IPLDLink,
    //pub mods: IPLDLink,
}
