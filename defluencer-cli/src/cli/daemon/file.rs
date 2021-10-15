use crate::actors::{Archivist, SetupAggregator, VideoAggregator};
use crate::server::start_server;
use crate::utils::config::Configuration;

use tokio::sync::mpsc::unbounded_channel;

use ipfs_api::IpfsClient;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct File {}

pub async fn file_cli(_file: File) {
    let ipfs = IpfsClient::default();

    if let Err(e) = ipfs.id(None).await {
        eprintln!("❗ IPFS must be started beforehand. {}", e);
        return;
    }

    println!("Initialization...");

    let config = match Configuration::from_file().await {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("❗ Configuration file not found. {}", e);
            return;
        }
    };

    let Configuration {
        input_socket_addr,
        mut archive,
        mut video,
        chat,
    } = config;

    let mut handles = Vec::with_capacity(4);

    let (archive_tx, archive_rx) = unbounded_channel();

    archive.archive_live_chat = false;

    let mut archivist = Archivist::new(ipfs.clone(), archive_rx);

    let archive_handle = tokio::spawn(async move {
        archivist.start().await;
    });

    handles.push(archive_handle);

    let (video_tx, video_rx) = unbounded_channel();

    video.pubsub_enable = false;

    let mut video = VideoAggregator::new(ipfs.clone(), video_rx, Some(archive_tx.clone()), video);

    let video_handle = tokio::spawn(async move {
        video.start().await;
    });

    handles.push(video_handle);

    let (setup_tx, setup_rx) = unbounded_channel();

    let mut setup = SetupAggregator::new(ipfs.clone(), setup_rx, video_tx.clone());

    let setup_handle = tokio::spawn(async move {
        setup.start().await;
    });

    handles.push(setup_handle);

    let server_handle = tokio::spawn(async move {
        start_server(
            input_socket_addr,
            video_tx,
            setup_tx,
            Some(archive_tx),
            ipfs,
            chat.topic,
        )
        .await;
    });

    handles.push(server_handle);

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("❗ Main: {}", e);
        }
    }
}
