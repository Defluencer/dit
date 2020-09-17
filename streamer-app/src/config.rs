use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub streamer_peer_id: String,
    pub gossipsub_topics: Topics,
    pub streamer_app: StreamerApp,
    pub variants: usize,
    pub video_segment_duration: usize,
    pub pin_stream: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamerApp {
    pub socket_addr: String,
    pub ffmpeg: Option<Ffmpeg>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ffmpeg {
    pub socket_addr: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Topics {
    pub video: String,
    pub chat: String,
}
