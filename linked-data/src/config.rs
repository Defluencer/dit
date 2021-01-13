use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub gossipsub_topics: Topics,
    pub addresses: Addrs,
    pub tracks: HashMap<String, Track>,
    pub segment_duration: usize,
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
pub struct Addrs {
    pub app_addr: String,
    pub ffmpeg_addr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub level: usize,
    pub codec: String,
    pub bandwidth: usize,
}

impl Default for Config {
    fn default() -> Self {
        let mut tracks = HashMap::new();

        tracks.insert(
            "1080p60".into(),
            Track {
                level: 3,
                codec: r#"video/mp4; codecs="avc1.42c02a, mp4a.40.2""#.into(),
                bandwidth: 6000000,
            },
        );

        tracks.insert(
            "720p30".into(),
            Track {
                level: 1,
                codec: r#"video/mp4; codecs="avc1.42c01f, mp4a.40.2""#.into(),
                bandwidth: 3000000,
            },
        );

        tracks.insert(
            "480p30".into(),
            Track {
                level: 0,
                codec: r#"video/mp4; codecs="avc1.42c01f, mp4a.40.2""#.into(),
                bandwidth: 2000000,
            },
        );

        tracks.insert(
            "720p60".into(),
            Track {
                level: 2,
                codec: r#"video/mp4; codecs="avc1.42c020, mp4a.40.2""#.into(),
                bandwidth: 4500000,
            },
        );

        Self {
            gossipsub_topics: Topics {
                video: "livelikevideo".into(),
                chat: "livelikechat".into(),
            },

            addresses: Addrs {
                app_addr: "127.0.0.1:2526".into(),
                ffmpeg_addr: Some("127.0.0.1:2525".into()),
            },

            tracks,

            segment_duration: 4,
        }
    }
}
