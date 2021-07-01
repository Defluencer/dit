use crate::cli::content::FEED_KEY;
use crate::cli::moderation::{BANS_KEY, MODS_KEY};
use crate::utils::config::Configuration;
use crate::utils::dag_nodes::{ipfs_dag_put_node_async, search_keypairs, update_ipns};
use serde::Serialize;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;
use ipfs_api::KeyType;

use linked_data::beacon::Topics;
use linked_data::feed::Feed;
use linked_data::moderation::{Bans, Moderators};

use structopt::StructOpt;

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
    /// GossipSub topic for live chat.
    #[structopt(long)]
    chat: String,

    /// GossipSub topic for video broadcasting.
    #[structopt(long)]
    videos: String,

    /// GossipSub topic for comments.
    #[structopt(long)]
    comments: String,
}

pub async fn beacon_cli(cli: Beacon) {
    let res = match cli.cmd {
        Command::Create(create) => create_beacon(create).await,
    };

    if let Err(e) = res {
        eprintln!("❗ IPFS: {}", e);
    }
}

async fn create_beacon(args: Create) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    let mut key_list = ipfs.key_list().await?;

    let bans = create_ipns_link::<Bans>(&ipfs, "Bans", BANS_KEY, &mut key_list).await?;
    let mods = create_ipns_link::<Moderators>(&ipfs, "Mods", MODS_KEY, &mut key_list).await?;
    let content_feed =
        create_ipns_link::<Feed>(&ipfs, "Content Feed", FEED_KEY, &mut key_list).await?;
    //let comments = create_ipns_link(&ipfs, "Comments", COMMENT_KEY, &mut key_list).await?;

    println!("Creating Beacon...");

    let mut config = match Configuration::from_file().await {
        Ok(conf) => conf,
        Err(_) => Configuration::default(),
    };

    config.chat.topic = args.chat;
    config.chat.mods = mods.clone();
    config.chat.bans = bans.clone();

    config.video.pubsub_topic = args.videos;

    config.save_to_file().await?;

    let topics = Topics {
        live_video: config.video.pubsub_topic,
        live_chat: config.chat.topic,
        comments: args.comments,
    };

    let res = ipfs.id(None).await?;
    let peer_id = res.id;

    #[cfg(debug_assertions)]
    println!("IPFS: peer id => {}", &peer_id);

    let beacon = linked_data::beacon::Beacon {
        topics,
        peer_id,
        bans,
        mods,
        content_feed,
        //comments,
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &beacon).await?;

    ipfs.pin_add(&cid.to_string(), true).await?;

    println!("✅ Beacon Created => ipfs://{}", &cid.to_string());

    Ok(())
}

async fn create_ipns_link<T>(
    ipfs: &IpfsClient,
    name: &str,
    key: &str,
    key_list: &mut ipfs_api::response::KeyPairList,
) -> Result<String, Error>
where
    T: Default + Serialize,
{
    let link = match search_keypairs(key, key_list) {
        Some(kp) => kp.id,
        None => {
            println!("Generating Key...");

            let key = generate_key(ipfs, key).await?;

            update_ipns(ipfs, &key, &T::default()).await?;

            key
        }
    };

    println!("✅ {} IPNS Link => {}", name, &link);

    Ok(link)
}

async fn generate_key(ipfs: &IpfsClient, key: &str) -> Result<String, Error> {
    let mut res = ipfs
        .key_gen(
            key,
            KeyType::Ed25519,
            64, /* Don't think this does anything... */
        )
        .await?;

    res.id.insert_str(0, "/ipns/"); // add this in front to make a path

    Ok(res.id)
}
