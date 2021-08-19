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
    /// Use either their beacon Cid OR their ethereum name service domain name
    Add(AddFriend),

    /// Remove a friend from your list.
    /// Use either their beacon Cid OR their ethereum name service domain name
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

    let (old_friends_cid, mut friends) = get_from_ipns::<Friendlies>(&ipfs, FRIENDS_KEY).await?;

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

    friends.list.insert(new_friend);

    update_ipns(&ipfs, FRIENDS_KEY, &friends).await?;
    ipfs.pin_rm(&old_friends_cid.to_string(), false).await?;

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct RemoveFriend {
    /// Beacon CID
    #[structopt(short, long)]
    beacon: Option<Cid>,

    /// Ethereum name service domain.
    #[structopt(short, long)]
    ens: Option<String>,
}

async fn remove_friend(command: RemoveFriend) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let RemoveFriend { beacon, ens } = command;

    let (old_friends_cid, mut friends) = get_from_ipns::<Friendlies>(&ipfs, FRIENDS_KEY).await?;

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

    friends.list.remove(&old_friend);

    update_ipns(&ipfs, FRIENDS_KEY, &friends).await?;
    ipfs.pin_rm(&old_friends_cid.to_string(), false).await?;

    Ok(())
}
