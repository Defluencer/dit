use crate::utils::dag_nodes::{get_from_ipns, update_ipns};

use hex::FromHex;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

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
    /// Manage list of banned users.
    Bans(BanCommands),

    /// Manage list of moderators.
    Mods(ModCommands),
}

pub async fn moderation_cli(cli: Moderation) {
    let res = match cli.cmd {
        Command::Bans(update) => ban_command(update).await,
        Command::Mods(update) => mod_command(update).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {}", e);
    }
}

#[derive(Debug, StructOpt)]
struct BanCommands {
    #[structopt(subcommand)]
    cmd: BanCommand,
}

#[derive(Debug, StructOpt)]
enum BanCommand {
    /// Ban users.
    Ban(Ban),

    /// Unban users.
    UnBan(UnBan),

    /// Replace the current list with another.
    ReplaceList(ReplaceBanList),
}

async fn ban_command(cli: BanCommands) -> Result<(), Error> {
    match cli.cmd {
        BanCommand::Ban(args) => ban_user(args).await,
        BanCommand::UnBan(args) => unban_user(args).await,
        BanCommand::ReplaceList(args) => replace_ban_list(args).await,
    }
}

#[derive(Debug, StructOpt)]
pub struct Ban {
    /// Ethereum Address.
    #[structopt(short, long)]
    address: String,
}

async fn ban_user(args: Ban) -> Result<(), Error> {
    let address = <[u8; 20]>::from_hex(&args.address).expect("Invalid Ethereum Adress");

    println!("Banning User...");

    let ipfs = IpfsClient::default();

    let mut ban_list: linked_data::moderation::Bans = get_from_ipns(&ipfs, BANS_KEY).await?;

    ban_list.banned.insert(address);

    update_ipns(&ipfs, BANS_KEY, &ban_list).await?;

    println!("✅ User {} Banned", args.address);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UnBan {
    /// Ethereum Address.
    #[structopt(short, long)]
    address: String,
}

async fn unban_user(args: UnBan) -> Result<(), Error> {
    let address = <[u8; 20]>::from_hex(&args.address).expect("Invalid Ethereum Adress");

    println!("Unbanning User...");

    let ipfs = IpfsClient::default();

    let mut ban_list: linked_data::moderation::Bans = get_from_ipns(&ipfs, BANS_KEY).await?;

    if ban_list.banned.remove(&address) {
        update_ipns(&ipfs, BANS_KEY, &ban_list).await?;

        println!("✅ User {} Unbanned", args.address);

        return Ok(());
    }

    println!("❗ User {} was not banned", args.address);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct ReplaceBanList {
    /// CID of the new ban list.
    #[structopt(long)]
    cid: Cid,
}

async fn replace_ban_list(args: ReplaceBanList) -> Result<(), Error> {
    println!("Replacing Ban List...");

    let ipfs = IpfsClient::default();

    ipfs.pin_add(&args.cid.to_string(), true).await?;

    ipfs.name_publish(&args.cid.to_string(), false, None, None, Some(BANS_KEY))
        .await?;

    println!(
        "✅ Previous Ban List Replaced with {}",
        &args.cid.to_string()
    );

    Ok(())
}

#[derive(Debug, StructOpt)]
struct ModCommands {
    #[structopt(subcommand)]
    cmd: ModCommand,
}

#[derive(Debug, StructOpt)]
enum ModCommand {
    /// Promote user to moderator.
    Mod(Mod),

    /// Demote user from moderator.
    UnMod(UnMod),

    /// Replace the current moderator list with another.
    ReplaceModList(ReplaceModList),
}

async fn mod_command(cli: ModCommands) -> Result<(), Error> {
    match cli.cmd {
        ModCommand::Mod(args) => mod_user(args).await,
        ModCommand::UnMod(args) => unmod_user(args).await,
        ModCommand::ReplaceModList(args) => replace_mod_list(args).await,
    }
}

#[derive(Debug, StructOpt)]
pub struct Mod {
    /// Ethereum address.
    #[structopt(long)]
    address: String,
}

async fn mod_user(args: Mod) -> Result<(), Error> {
    let address = <[u8; 20]>::from_hex(&args.address).expect("Invalid Ethereum Adress");

    println!("Promoting User...");

    let ipfs = IpfsClient::default();

    let mut mods_list: linked_data::moderation::Moderators = get_from_ipns(&ipfs, MODS_KEY).await?;

    mods_list.mods.insert(address);

    update_ipns(&ipfs, MODS_KEY, &mods_list).await?;

    println!("✅ User {} Promoted To Moderator", args.address);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UnMod {
    /// Ethereum address.
    #[structopt(long)]
    address: String,
}

async fn unmod_user(args: UnMod) -> Result<(), Error> {
    let address = <[u8; 20]>::from_hex(&args.address).expect("Invalid Ethereum Adress");

    println!("Demoting Moderator...");

    let ipfs = IpfsClient::default();

    let mut mod_list: linked_data::moderation::Moderators = get_from_ipns(&ipfs, MODS_KEY).await?;

    if mod_list.mods.remove(&address) {
        update_ipns(&ipfs, MODS_KEY, &mod_list).await?;

        println!("✅ Moderator {} Demoted", args.address);

        return Ok(());
    }

    println!("❗ User {} Was Not A Moderator", args.address);

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct ReplaceModList {
    /// CID of the new moderator list
    #[structopt(long)]
    cid: Cid,
}

async fn replace_mod_list(args: ReplaceModList) -> Result<(), Error> {
    println!("Replacing Moderator List...");

    let ipfs = IpfsClient::default();

    ipfs.pin_add(&args.cid.to_string(), true).await?;

    ipfs.name_publish(&args.cid.to_string(), false, None, None, Some(MODS_KEY))
        .await?;

    println!(
        "✅ Previous Moderator List Replaced with {}",
        &args.cid.to_string()
    );

    Ok(())
}
