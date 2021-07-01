mod actors;
mod cli;
mod server;
mod utils;

use crate::cli::beacon::{beacon_cli, Beacon};
use crate::cli::content::{content_feed_cli, ContentFeed};
use crate::cli::file::{file_cli, File};
use crate::cli::moderation::{moderation_cli, Moderation};
use crate::cli::stream::{stream_cli, Stream};

use structopt::StructOpt;

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

    /// Appoint moderators & ban or unban users.
    Moderation(Moderation),

    /// Manage your content feed.
    ContentFeed(ContentFeed),
}

#[tokio::main]
async fn main() {
    match CommandLineInterface::from_args() {
        CommandLineInterface::Stream(stream) => stream_cli(stream).await,
        CommandLineInterface::File(file) => file_cli(file).await,
        CommandLineInterface::Beacon(beacon) => beacon_cli(beacon).await,
        CommandLineInterface::Moderation(mods) => moderation_cli(mods).await,
        CommandLineInterface::ContentFeed(feed) => content_feed_cli(feed).await,
    }
}
