use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub gossipsub_topics: Topics,
    pub addresses: Addrs,
    pub segment_duration: usize,
    //pub blacklist: IPLDLink,
    //pub whitelist: IPLDLink,
    //pub mods: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Topics {
    pub live_video: String,
    pub live_chat: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Addrs {
    pub app_addr: String,
    pub ffmpeg_addr: Option<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            gossipsub_topics: Topics {
                live_video: "defluencer_live_video".into(),
                live_chat: "defluencer_live_chat".into(),
            },

            addresses: Addrs {
                app_addr: "127.0.0.1:2526".into(),
                ffmpeg_addr: Some("127.0.0.1:2525".into()),
            },

            segment_duration: 4,
        }
    }
}
