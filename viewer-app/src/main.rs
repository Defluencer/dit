mod playlist;
mod pubsub;
mod services;

use crate::playlist::Playlist;
use crate::pubsub::pubsub_sub;
use crate::services::get_requests;

use futures_util::future::join;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Error, Server};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, RwLock};

const SERVER_PORT: u16 = 2525;

const PUBSUB_TOPIC_VIDEO: &str = "live_like_video";

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Viewer Application Initialization...");

    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

    let playlist = Arc::new(RwLock::new(Playlist::new(3, 4, 5)));

    let playlist_clone = playlist.clone();

    let make_service = make_service_fn(move |_| {
        //TODO try understand this mess...

        let playlist_clone = playlist_clone.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let playlist_clone = playlist_clone.clone();

                async move { get_requests(req, playlist_clone).await }
            }))
        }
    });

    let server = Server::bind(&server_addr).serve(make_service);

    let (_, result) = join(pubsub_sub(PUBSUB_TOPIC_VIDEO, playlist), server).await;

    result
}
