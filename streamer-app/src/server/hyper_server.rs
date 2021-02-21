use crate::actors::{Archive, VideoData};
use crate::server::services::put_requests;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;

use tokio::signal::ctrl_c;
use tokio::sync::mpsc::Sender;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use ipfs_api::IpfsClient;

async fn shutdown_signal(archive_tx: Sender<Archive>) {
    ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");

    let msg = Archive::Finalize;

    if let Err(error) = archive_tx.send(msg).await {
        eprintln!("Archive receiver hung up {}", error);
    }
}

pub async fn start_server(
    server_addr: String,
    collector: Sender<VideoData>,
    archive_tx: Sender<Archive>,
    ipfs: IpfsClient,
) {
    let server_addr = SocketAddr::from_str(&server_addr).expect("Invalid server address");

    let service = make_service_fn(move |_| {
        let ipfs = ipfs.clone();
        let collector = collector.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                put_requests(req, collector.clone(), ipfs.clone())
            }))
        }
    });

    let server = Server::bind(&server_addr)
        .http1_half_close(true)
        .serve(service);

    println!("Ingess Server Online");

    let graceful = server.with_graceful_shutdown(shutdown_signal(archive_tx));

    if let Err(e) = graceful.await {
        eprintln!("Server error {}", e);
    }
}
