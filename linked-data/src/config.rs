use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub input_socket_addrs: String,
    pub archive: Option<ArchiveConfig>,
    pub video: VideoConfig,
    pub chat: ChatConfig,
    pub ffmpeg: Option<FFMPEGConfig>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            input_socket_addrs: "127.0.0.1:2526".into(),

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

            ffmpeg: Some(FFMPEGConfig {
                output_socket_addrs: "localhost:2526".into(),
                input_socket_addrs: Some("localhost:2525".into()),
            }),
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

#[derive(Serialize, Deserialize, Debug)]
pub struct FFMPEGConfig {
    pub output_socket_addrs: String,
    pub input_socket_addrs: Option<String>,
}
