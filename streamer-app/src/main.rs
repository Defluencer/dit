mod collector;
mod ffmpeg_transcoding;
mod server;
mod services;

use tokio::sync::mpsc::channel;

use ipfs_api::IpfsClient;

#[tokio::main]
async fn main() {
    println!("Streamer Application Initialization...");

    let ipfs = IpfsClient::default();

    let (tx, rx) = channel(4);

    tokio::join!(
        collector::collect_video_data(ipfs, rx),
        server::start_server(tx),
        ffmpeg_transcoding::start()
    );
}
