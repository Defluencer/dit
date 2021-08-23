use std::convert::TryFrom;

use crate::cli::content::{COMMENTS_KEY, FEED_KEY};
use crate::cli::friends::FRIENDS_KEY;
use crate::cli::moderation::{BANS_KEY, MODS_KEY};
use crate::utils::config::Configuration;
use crate::utils::dag_nodes::{ipfs_dag_put_node_async, search_keypairs, update_ipns};

use serde::Serialize;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;
use ipfs_api::KeyType;

use linked_data::beacon::Topics;
use linked_data::comments::Commentary;
use linked_data::feed::FeedAnchor;
use linked_data::friends::Friendlies;
use linked_data::keccak256;
use linked_data::moderation::{Bans, Moderators};

use structopt::StructOpt;

use cid::multibase::{encode, Base};
use cid::Cid;

#[derive(Debug, StructOpt)]
pub struct Beacon {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Create a new beacon.
    Create(Create),
}

#[derive(Debug, StructOpt)]
pub struct Create {
    /// Your choosen display name.
    #[structopt(short, long)]
    display_name: String,
}

pub async fn beacon_cli(cli: Beacon) {
    let res = match cli.cmd {
        Command::Create(create) => create_beacon(create).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {:#?}", e);
    }
}

async fn create_beacon(args: Create) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let Create { display_name } = args;

    let key_list = ipfs.key_list().await?;

    let (bans, mods, content_feed, comments, friends) = tokio::try_join!(
        create_ipns_link::<Bans>(&ipfs, "Bans", BANS_KEY, &key_list),
        create_ipns_link::<Moderators>(&ipfs, "Mods", MODS_KEY, &key_list),
        create_ipns_link::<FeedAnchor>(&ipfs, "Content Feed", FEED_KEY, &key_list),
        create_ipns_link::<Commentary>(&ipfs, "Comments", COMMENTS_KEY, &key_list),
        create_ipns_link::<Friendlies>(&ipfs, "Friends", FRIENDS_KEY, &key_list)
    )?;

    println!("Creating Beacon...");

    let mut config = match Configuration::from_file().await {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("❗ Configuration: {:#?}", e);
            Configuration::default()
        }
    };

    config.chat.topic = encode(
        Base::Base32Lower,
        &keccak256(&format!("{}_video", &display_name).into_bytes()),
    );
    config.video.pubsub_topic = encode(
        Base::Base32Lower,
        &keccak256(&format!("{}_chat", &display_name).into_bytes()),
    );

    config.save_to_file().await?;

    let topics = Topics {
        video: config.video.pubsub_topic,
        chat: config.chat.topic,
    };

    let res = ipfs.id(None).await?;
    let peer_id = res.id;

    #[cfg(debug_assertions)]
    println!("IPFS: peer id => {}", &peer_id);

    let beacon = linked_data::beacon::Beacon {
        topics,
        peer_id,
        display_name,
        bans: Some(bans),
        mods: Some(mods),
        content_feed,
        comments: Some(comments),
        friends: Some(friends),
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &beacon).await?;

    ipfs.pin_add(&cid.to_string(), false).await?;

    println!("✅ Beacon Created => {:?}", &cid);

    Ok(())
}

async fn create_ipns_link<T>(
    ipfs: &IpfsClient,
    name: &str,
    key: &str,
    key_list: &ipfs_api::response::KeyPairList,
) -> Result<Cid, Error>
where
    T: Default + Serialize,
{
    let link = match search_keypairs(key, key_list) {
        Some(kp) => kp.id.to_owned(),
        None => {
            println!("Generating {} IPNS Key...", name);

            let ipns_link = generate_key(ipfs, key).await?;

            println!("Updating {} IPNS Link...", name);

            update_ipns(ipfs, key, &T::default()).await?;

            ipns_link
        }
    };

    let cid = Cid::try_from(link).expect("Serialize CID");

    println!("✅ {} IPNS Link => {}", name, &cid);

    Ok(cid)
}

async fn generate_key(ipfs: &IpfsClient, key: &str) -> Result<String, Error> {
    let res = ipfs
        .key_gen(
            key,
            KeyType::Ed25519,
            64, /* Don't think this does anything... */
        )
        .await?;

    Ok(res.id)
}
