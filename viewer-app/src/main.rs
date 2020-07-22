mod playlist;
mod pubsub;
mod server;
mod services;

use std::sync::Arc;
use std::sync::RwLock;

use crate::playlist::Playlists;
use crate::pubsub::pubsub_sub;
use crate::server::start_server;

#[tokio::main]
async fn main() {
    println!("Viewer Application Initialization...");

    let playlist = Arc::new(RwLock::new(Playlists::new()));

    tokio::join!(start_server(playlist.clone()), pubsub_sub(playlist));
}
