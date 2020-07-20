use std::sync::mpsc::channel;

use notify::{raw_watcher, RecursiveMode, Watcher};

use ipfs_api::IpfsClient;

//mod ffmpeg_transcoding;
mod file_watcher;

const LOCAL_FOLDER: &str = "./";

#[tokio::main]
async fn main() {
    println!("Streamer Application Initialization...");

    let client = IpfsClient::default();

    let (tx, rx) = channel();

    //Raw watcher is used to minimize latency,
    //it work well with ffmpeg option to write a .tmp file first then
    //rename it when done writing.
    let mut watcher = raw_watcher(tx).expect("Can't start file watcher.");

    watcher
        .watch(LOCAL_FOLDER, RecursiveMode::Recursive)
        .expect("Can't watch folder.");

    println!("File Watcher Started! Do not modify files while the process is running.");

    //tokio::join!(ffmpeg_transcoding::start(), file_watcher::start(rx, client));

    file_watcher::start(rx, client).await;
}
