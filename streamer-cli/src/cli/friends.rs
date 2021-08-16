use crate::utils::dag_nodes::{
    get_from_ipns, ipfs_dag_get_node_async, ipfs_dag_put_node_async, update_ipns,
};

use serde::de::DeserializeOwned;
use serde::Serialize;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::comments::Commentary;
use linked_data::feed::FeedAnchor;
use linked_data::video::{DayNode, HourNode, MinuteNode, VideoMetadata};

use cid::Cid;

use structopt::StructOpt;

pub const FRIENDS_KEY: &str = "friends";

#[derive(Debug, StructOpt)]
pub struct Friends {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Publish new content to your feed.
    Add(AddFriend),

    /// Delete content from your feed.
    Remove(RemoveFriend),
}

pub async fn friends_cli(cli: Friends) {
    let res = match cli.cmd {
        Command::Add(add) => add_friend(add).await,
        Command::Remove(remove) => remove_friend(remove).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {:#?}", e);
    }
}

#[derive(Debug, StructOpt)]
pub struct AddFriend {
    /// ENS domain name
    #[structopt(short, long)]
    name: String,

    /// Beacon CID
    #[structopt(short, long)]
    content: Cid,
}

async fn add_friend(command: AddFriend) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    //TODO

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct RemoveFriend {
    /// ENS domain name
    #[structopt(short, long)]
    name: String,

    /// Beacon CID
    #[structopt(short, long)]
    content: Cid,
}

async fn remove_friend(command: RemoveFriend) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    //TODO

    Ok(())
}
