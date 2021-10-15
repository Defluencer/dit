use crate::utils::dag_nodes::{get_from_ipns, update_ipns};

//use std::path::PathBuf;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::identity::Identity;

use cid::Cid;

use structopt::StructOpt;

pub const IDENTITY_KEY: &str = "identity";

#[derive(Debug, StructOpt)]
pub struct IdentityCLI {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Choose a new display name.
    Name(UpdateName),

    /// Choose a new image avatar.
    Avatar(UpdateAvatar),
}

pub async fn identity_cli(cli: IdentityCLI) {
    let res = match cli.cmd {
        Command::Name(name) => update_name(name).await,
        Command::Avatar(avatar) => update_avatar(avatar).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {:#?}", e);
    }
}

#[derive(Debug, StructOpt)]
pub struct UpdateName {
    /// Display name.
    #[structopt(short, long)]
    name: String,
}

async fn update_name(command: UpdateName) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let UpdateName { name } = command;

    let (old_id_cid, mut id) = get_from_ipns::<Identity>(&ipfs, IDENTITY_KEY).await?;

    id.display_name = name;

    update_ipns(&ipfs, IDENTITY_KEY, &id).await?;

    let oidc = old_id_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&oidc, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", oidc, e);
    }

    println!("✅ Display Name Updated");

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct UpdateAvatar {
    /// Link to image avatar.
    #[structopt(short, long)]
    image: Cid,
    // Path to image file.
    //#[structopt(short, long)]
    //path: Option<PathBuf>,
}

async fn update_avatar(command: UpdateAvatar) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let UpdateAvatar { image } = command;

    let (old_id_cid, mut id) = get_from_ipns::<Identity>(&ipfs, IDENTITY_KEY).await?;

    id.avatar = image.into();

    update_ipns(&ipfs, IDENTITY_KEY, &id).await?;

    let ofc = old_id_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&ofc, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", ofc, e);
    }

    println!("✅ Avatar Updated");

    Ok(())
}
