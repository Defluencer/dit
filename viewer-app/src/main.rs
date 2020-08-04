mod playlist;
mod pubsub;
mod server;
mod services;

use std::sync::Arc;

use tokio::sync::RwLock;

use ipfs_api::IpfsClient;

use crate::playlist::Playlists;
use crate::pubsub::{pubsub_sub, PUBSUB_TOPIC_VIDEO, STREAMER_PEER_ID};
use crate::server::start_server;

#[tokio::main]
async fn main() {
    println!("Viewer Application Initialization...");

    let ipfs = IpfsClient::default();

    match ipfs.config("Identity.PeerID", None, None, None).await {
        Ok(peer_id) => {
            println!("Viewer: {}", peer_id.value);
            println!("Streamer: {}", STREAMER_PEER_ID);
            println!("Topic: {}", PUBSUB_TOPIC_VIDEO);
        }
        Err(_) => {
            eprintln!("Error! Is IPFS running with PubSub enabled?");
            return;
        }
    }

    let playlist = Arc::new(RwLock::new(Playlists::new()));

    tokio::join!(pubsub_sub(playlist.clone(), ipfs), start_server(playlist));
}
