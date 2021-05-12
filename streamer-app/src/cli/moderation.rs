use crate::cli::beacon::search_keypairs;
use crate::utils::dag_nodes::{ipfs_dag_get_node_async, ipfs_dag_put_node_async};

use std::convert::TryFrom;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::video::{DayNode, HourNode, MinuteNode, VideoList, VideoMetadata};
use linked_data::IPLDLink;

use cid::Cid;

use structopt::StructOpt;

pub const BANS_KEY: &str = "bans";
pub const MODS_KEY: &str = "mods";

#[derive(Debug, StructOpt)]
pub struct Moderation {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// TODO
    Bans(UpdateBans),

    /// TODO
    Mods(UpdateMods),
}

#[derive(Debug, StructOpt)]
pub struct UpdateBans {
    /// TODO
    #[structopt(short, long, default_value = BANS_KEY)]
    key: String,

    /// TODO
    #[structopt(long)]
    cid: Cid,
}

#[derive(Debug, StructOpt)]
pub struct UpdateMods {
    /// TODO
    #[structopt(short, long, default_value = MODS_KEY)]
    key: String,

    /// TODO
    #[structopt(long)]
    cid: Cid,
}

pub async fn moderation_cli(cli: Moderation) {
    let res = match cli.cmd {
        Command::Bans(update) => update_ban_list(update).await,
        Command::Mods(update) => update_mod_list(update).await,
    };

    if let Err(e) = res {
        eprintln!("IPFS: {}", e);
    }
}

async fn update_ban_list(command: UpdateBans) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    //TODO

    Ok(())
}

async fn update_mod_list(command: UpdateMods) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    //TODO

    Ok(())
}

/// Serialize the new bans list, pin it then publish it under this IPNS key.
pub async fn update_bans_list(
    ipfs: &IpfsClient,
    key: &str,
    bans_list: &linked_data::moderation::Bans,
) -> Result<(), Error> {
    println!("Updating Bans List...");

    let cid = ipfs_dag_put_node_async(ipfs, bans_list).await?;

    ipfs.pin_add(&cid.to_string(), true).await?;

    ipfs.name_publish(&cid.to_string(), false, None, None, Some(key))
        .await?;

    println!("New Bans List CID => {}", &cid.to_string());

    Ok(())
}

/// Serialize the new mods list, pin it then publish it under this IPNS key.
pub async fn update_mods_list(
    ipfs: &IpfsClient,
    key: &str,
    bans_list: &linked_data::moderation::Moderators,
) -> Result<(), Error> {
    println!("Updating Mods List...");

    let cid = ipfs_dag_put_node_async(ipfs, bans_list).await?;

    ipfs.pin_add(&cid.to_string(), true).await?;

    ipfs.name_publish(&cid.to_string(), false, None, None, Some(key))
        .await?;

    println!("New Mods List CID => {}", &cid.to_string());

    Ok(())
}
