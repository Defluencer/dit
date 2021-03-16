mod actors;
mod beacon;
mod file;
mod server;
mod stream;
mod utils;

use crate::beacon::create_beacon;
use crate::file::start_file;
use crate::stream::start_stream;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about)]
#[structopt(rename_all = "kebab-case")]
enum CommandLineInterface {
    /// Start live streaming daemon.
    Stream(Stream),

    /// Start file streaming daemon.
    File(File),

    /// Create content beacon.
    Beacon(Beacon),
}

#[derive(Debug, StructOpt)]
pub struct Stream {
    /// Disable chat archiving fonctionalities.
    #[structopt(long)]
    no_chat: bool,

    /// Disable all archiving.
    #[structopt(long)]
    no_archive: bool,
}

#[derive(Debug, StructOpt)]
pub struct File {}

#[derive(Debug, StructOpt)]
pub struct Beacon {
    /// GossipSub topic for receiving chat messages.
    #[structopt(long)]
    chat_topic: String,

    /// GossipSub topic for video broadcasting.
    #[structopt(long)]
    video_topic: String,

    /// IPNS key name for video list resolution.
    #[structopt(long, default_value = "videolist")]
    key_name: String,
}

#[tokio::main]
async fn main() {
    match CommandLineInterface::from_args() {
        CommandLineInterface::Stream(stream) => start_stream(stream).await,
        CommandLineInterface::File(file) => start_file(file).await,
        CommandLineInterface::Beacon(beacon) => create_beacon(beacon).await,
    }
}
