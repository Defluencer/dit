use crate::utils::dag_nodes::{get_from_ipns, update_ipns};

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use linked_data::friends::{Friend, Friendlies};

use cid::Cid;

use structopt::StructOpt;

use either::Either;

pub const FRIENDS_KEY: &str = "friends";

#[derive(Debug, StructOpt)]
pub struct Friends {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Add a new friend to your list.
    /// Use either their beacon Cid OR their ethereum name service domain name.
    Add(AddFriend),

    /// Remove a friend from your list.
    /// Use either their beacon Cid OR their ethereum name service domain name.
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
    /// Beacon CID.
    #[structopt(short, long)]
    beacon: Option<Cid>,

    /// Ethereum name service domain.
    #[structopt(short, long)]
    ens: Option<String>,
}

async fn add_friend(command: AddFriend) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let AddFriend { beacon, ens } = command;

    let new_friend = match (beacon, ens) {
        (Some(cid), None) => Friend {
            friend: Either::Right(cid.into()),
        },
        (None, Some(name)) => Friend {
            friend: Either::Left(name),
        },
        (_, _) => {
            return Err(Error::Uncategorized(
                "Use either beacon Cid Or ENS domain name".into(),
            ))
        }
    };

    println!("Adding Friend {:?}", &new_friend.friend);

    let (old_friends_cid, mut list) = get_from_ipns::<Friendlies>(&ipfs, FRIENDS_KEY).await?;

    list.friends.insert(new_friend);

    println!("Updating Friends List...");

    update_ipns(&ipfs, FRIENDS_KEY, &list).await?;

    println!("Unpinning Old List...");

    let ofc = old_friends_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&ofc, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", ofc, e);
    }

    println!("✅ Friend Added");

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct RemoveFriend {
    /// Beacon CID
    #[structopt(short, long)]
    beacon: Option<Cid>,

    /// Ethereum name service domain name.
    #[structopt(short, long)]
    ens: Option<String>,
}

async fn remove_friend(command: RemoveFriend) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let RemoveFriend { beacon, ens } = command;

    let old_friend = match (beacon, ens) {
        (Some(cid), None) => Friend {
            friend: Either::Right(cid.into()),
        },
        (None, Some(name)) => Friend {
            friend: Either::Left(name),
        },
        (_, _) => {
            return Err(Error::Uncategorized(
                "Use either beacon Cid Or ENS domain name".into(),
            ))
        }
    };

    println!("Removing Friend {:?}", &old_friend.friend);

    let (old_friends_cid, mut list) = get_from_ipns::<Friendlies>(&ipfs, FRIENDS_KEY).await?;

    list.friends.remove(&old_friend);

    println!("Updating Friends List...");

    update_ipns(&ipfs, FRIENDS_KEY, &list).await?;

    println!("Unpinning Old List...");

    let ofc = old_friends_cid.to_string();
    if let Err(e) = ipfs.pin_rm(&ofc, false).await {
        eprintln!("❗ IPFS could not unpin {}. Error: {}", ofc, e);
    }

    println!("✅ Friend Removed");

    Ok(())
}
