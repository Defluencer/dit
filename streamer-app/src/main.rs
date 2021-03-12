mod actors;
mod server;
mod utils;

use std::env;
use std::path::PathBuf;

use crate::actors::{
    file_transcoding, stream_transcoding, Archivist, ChatAggregator, VideoAggregator,
};
use crate::server::start_server;
use crate::utils::get_config;

use tokio::sync::mpsc::unbounded_channel;

use ipfs_api::IpfsClient;

use linked_data::config::{Configuration, FFMPEGConfig};

//cli
//streamer-app --help
//streamer-app --file=<File>
//streamer-app --stream=<SocketAddress>
//streamer-app --ffmpeg
//streamer-app --archive

#[tokio::main]
async fn main() {
    println!("Initialization...");

    let ipfs = IpfsClient::default();

    let config = get_config().await;

    let Configuration {
        input_socket_addrs,
        archive,
        video,
        chat,
        ffmpeg,
    } = config;

    let mut handles = Vec::with_capacity(4);

    let archive_tx = {
        if let Some(archive_config) = archive {
            let (archive_tx, archive_rx) = unbounded_channel();

            if archive_config.archive_live_chat {
                let mut chat = ChatAggregator::new(ipfs.clone(), archive_tx.clone(), chat);

                let chat_handle = tokio::spawn(async move {
                    chat.start_receiving().await;
                });

                handles.push(chat_handle);
            }

            let mut archivist = Archivist::new(ipfs.clone(), archive_rx, archive_config);

            let archive_handle = tokio::spawn(async move {
                archivist.collect().await;
            });

            handles.push(archive_handle);

            Some(archive_tx)
        } else {
            None
        }
    };

    let (video_tx, video_rx) = unbounded_channel();

    let mut video = VideoAggregator::new(ipfs.clone(), video_rx, archive_tx.clone(), video);

    let video_handle = tokio::spawn(async move {
        video.start_receiving().await;
    });

    handles.push(video_handle);

    let input_socket_addrs_clone = input_socket_addrs.clone();

    let server_handle = tokio::spawn(async move {
        start_server(input_socket_addrs_clone, video_tx, archive_tx, ipfs).await;
    });

    handles.push(server_handle);

    if let Some(ffmpeg_config) = ffmpeg {
        let FFMPEGConfig {
            input_socket_addrs,
            output_socket_addrs,
        } = ffmpeg_config;

        let ffmpeg_handle = match input_socket_addrs {
            Some(input) => tokio::spawn(async move {
                stream_transcoding(input, output_socket_addrs).await;
            }),
            None => {
                let mut args: Vec<String> = env::args().collect();

                let path = PathBuf::from(args.pop().unwrap());

                tokio::spawn(async move {
                    file_transcoding(path, output_socket_addrs).await;
                })
            }
        };

        handles.push(ffmpeg_handle);
    }

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Main: {}", e);
        }
    }
}
