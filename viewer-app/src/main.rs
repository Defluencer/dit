mod playlist;
mod pubsub;
mod server;
mod services;

use std::sync::Arc;
use std::sync::RwLock;

use crate::playlist::Playlist;
use crate::pubsub::pubsub_sub;
use crate::server::start_server;

const PUBSUB_TOPIC_VIDEO: &str = "live_like_video";

#[tokio::main]
async fn main() {
    println!("Viewer Application Initialization...");

    let playlist_1080_60 = Arc::new(RwLock::new(Playlist::new(3, 4, 5)));
    let playlist_720_60 = Arc::new(RwLock::new(Playlist::new(3, 4, 5)));
    let playlist_720_30 = Arc::new(RwLock::new(Playlist::new(3, 4, 5)));
    let playlist_480_30 = Arc::new(RwLock::new(Playlist::new(3, 4, 5)));

    let fut_1080_60 = pubsub_sub(PUBSUB_TOPIC_VIDEO, playlist_1080_60);
    let fut_720_60 = pubsub_sub(PUBSUB_TOPIC_VIDEO, playlist_720_60);
    let fut_720_30 = pubsub_sub(PUBSUB_TOPIC_VIDEO, playlist_720_30);
    let fut_480_30 = pubsub_sub(PUBSUB_TOPIC_VIDEO, playlist_480_30);

    tokio::join!(
        fut_1080_60,
        fut_720_60,
        fut_720_30,
        fut_480_30,
        start_server()
    );
}
