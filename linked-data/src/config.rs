use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub addresses: Addrs,
    pub archive: Option<ArchiveConfig>,
    pub video: VideoConfig,
    pub chat: ChatConfig,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            addresses: Addrs {
                app_addr: "127.0.0.1:2526".into(),
                ffmpeg_addr: Some("127.0.0.1:2525".into()),
            },

            archive: Some(ArchiveConfig {
                archive_live_chat: true,
                segment_duration: 4,
            }),

            video: VideoConfig {
                pubsub_topic: Some("defluencer_live_video".into()),
            },

            chat: ChatConfig {
                pubsub_topic: "defluencer_live_chat".into(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Addrs {
    pub app_addr: String,
    pub ffmpeg_addr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArchiveConfig {
    pub archive_live_chat: bool,
    pub segment_duration: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoConfig {
    pub pubsub_topic: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatConfig {
    pub pubsub_topic: String,
    //pub blacklist: IPLDLink,
    //pub whitelist: IPLDLink,
    //pub mods: IPLDLink,
}
