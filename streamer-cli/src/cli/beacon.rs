use crate::cli::moderation::{BANS_KEY, MODS_KEY};
use crate::cli::video::VIDEOS_KEY;
use crate::utils::config::Configuration;
use crate::utils::dag_nodes::{ipfs_dag_put_node_async, search_keypairs, update_ipns};

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;
use ipfs_api::KeyType;

use linked_data::beacon::Topics;
use linked_data::moderation::{Bans, Moderators};
use linked_data::video::VideoList;

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
    #[structopt(short, long)]
    chat_topic: String,

    /// GossipSub topic for video broadcasting.
    #[structopt(short, long)]
    video_topic: String,
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

    let mut res = ipfs.key_list().await?;

    let videos = match search_keypairs(&VIDEOS_KEY, &mut res) {
        Some(kp) => kp.id,
        None => {
            println!("Generating Key...");

            let key = generate_key(&ipfs, &VIDEOS_KEY).await?;

            update_ipns(&ipfs, &VIDEOS_KEY, &VideoList::default()).await?;

            key
        }
    };

    println!("✅ Videos IPNS Link => {}", &videos);

    let bans = match search_keypairs(&BANS_KEY, &mut res) {
        Some(kp) => kp.id,
        None => {
            println!("Generating Key...");

            let key = generate_key(&ipfs, &BANS_KEY).await?;

            update_ipns(&ipfs, &BANS_KEY, &Bans::default()).await?;

            key
        }
    };

    println!("✅ Bans IPNS Link => {}", &bans);

    let mods = match search_keypairs(&MODS_KEY, &mut res) {
        Some(kp) => kp.id,
        None => {
            println!("Generating Key...");

            let key = generate_key(&ipfs, &MODS_KEY).await?;

            update_ipns(&ipfs, &MODS_KEY, &Moderators::default()).await?;

            key
        }
    };

    println!("✅ Mods IPNS Link => {}", &mods);

    println!("Creating Beacon...");

    let mut config = match Configuration::from_file().await {
        Ok(conf) => conf,
        Err(_) => Configuration::default(),
    };

    config.chat.topic = args.chat_topic;
    config.chat.mods = mods.clone();
    config.chat.bans = bans.clone();

    config.video.pubsub_topic = args.video_topic;

    config.save_to_file().await?;

    let topics = Topics {
        live_video: config.video.pubsub_topic,
        live_chat: config.chat.topic,
    };

    let res = ipfs.id(None).await?;
    let peer_id = res.id;

    #[cfg(debug_assertions)]
    println!("IPFS: peer id => {}", &peer_id);

    let beacon = linked_data::beacon::Beacon {
        topics,
        peer_id,
        videos,
        bans,
        mods,
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &beacon).await?;

    ipfs.pin_add(&cid.to_string(), true).await?;

    println!("✅ Beacon Created => ipfs://{}", &cid.to_string());

    Ok(())
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
