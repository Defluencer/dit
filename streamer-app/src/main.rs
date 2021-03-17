mod actors;
mod beacon;
mod file;
mod server;
mod stream;
mod utils;
mod video;

use crate::beacon::{beacon_cli, Beacon};
use crate::file::{file_cli, File};
use crate::stream::{stream_cli, Stream};
use crate::video::{video_cli, Video};

use structopt::StructOpt;

const DEFAULT_KEY: &str = "videolist";

#[derive(Debug, StructOpt)]
#[structopt(about)]
#[structopt(rename_all = "kebab-case")]
enum CommandLineInterface {
    /// Start the live streaming daemon.
    Stream(Stream),

    /// Start the file streaming daemon.
    File(File),

    /// Create a content beacon.
    Beacon(Beacon),

    /// Create, update and delete videos.
    Video(Video),
}

#[tokio::main]
async fn main() {
    match CommandLineInterface::from_args() {
        CommandLineInterface::Stream(stream) => stream_cli(stream).await,
        CommandLineInterface::File(file) => file_cli(file).await,
        CommandLineInterface::Beacon(beacon) => beacon_cli(beacon).await,
        CommandLineInterface::Video(video) => video_cli(video).await,
    }
}
