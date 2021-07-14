use crate::actors::{Archive, SetupData, VideoData};
use crate::server::services::put_requests;

use std::convert::Infallible;
use std::net::SocketAddr;

use tokio::signal::ctrl_c;
use tokio::sync::mpsc::UnboundedSender;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use ipfs_api::IpfsClient;

async fn shutdown_signal(
    ipfs: IpfsClient,
    topic: String,
    archive_tx: Option<UnboundedSender<Archive>>,
) {
    ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");

    if let Some(archive_tx) = archive_tx {
        let msg = Archive::Finalize;

        if let Err(error) = archive_tx.send(msg) {
            eprintln!("Archive receiver hung up {}", error);
        }

        //Hacky way to shutdown chat actor. Send some msg to trigger a check
        ipfs.pubsub_pub(&topic, "Stopping")
            .await
            .expect("PubSub Pub Failed!");
    }
}

pub async fn start_server(
    server_addr: SocketAddr,
    video_tx: UnboundedSender<VideoData>,
    setup_tx: UnboundedSender<SetupData>,
    archive_tx: Option<UnboundedSender<Archive>>,
    ipfs: IpfsClient,
    topic: String,
) {
    let ipfs_clone = ipfs.clone();

    let service = make_service_fn(move |_| {
        let ipfs = ipfs.clone();
        let video_tx = video_tx.clone();
        let setup_tx = setup_tx.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                put_requests(req, video_tx.clone(), setup_tx.clone(), ipfs.clone())
            }))
        }
    });

    let server = Server::bind(&server_addr)
        .http1_half_close(true) //FFMPEG requirement
        .serve(service);

    println!("✅ Ingess Server Online");

    let graceful = server.with_graceful_shutdown(shutdown_signal(ipfs_clone, topic, archive_tx));

    if let Err(e) = graceful.await {
        eprintln!("Server: {}", e);
    }

    println!("❌ Ingess Server Offline");
}
