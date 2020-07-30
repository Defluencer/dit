mod playlist;
mod pubsub;
mod server;
mod services;

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::playlist::Playlists;
use crate::pubsub::pubsub_sub;
use crate::server::start_server;

#[tokio::main]
async fn main() {
    println!("Viewer Application Initialization...");

    let playlist = Arc::new(RwLock::new(Playlists::new()));

    tokio::join!(pubsub_sub(playlist.clone()), start_server(playlist));
}
