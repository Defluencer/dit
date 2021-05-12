use crate::cli::moderation::{update_bans_list, update_mods_list, BANS_KEY, MODS_KEY};
use crate::cli::video::update_video_list;
use crate::cli::video::VIDEOS_KEY;
use crate::utils::config::{get_config, set_config};
use crate::utils::dag_nodes::ipfs_dag_put_node_async;

use ipfs_api::response::{Error, KeyListResponse, KeyPair};
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

    /// IPNS key name of link to videos.
    #[structopt(long, default_value = VIDEOS_KEY)]
    videos_key: String,

    /// IPNS key name of link to ban list.
    #[structopt(long, default_value = BANS_KEY)]
    bans_key: String,

    /// IPNS key name of link to ban list.
    #[structopt(long, default_value = MODS_KEY)]
    mods_key: String,
}

pub async fn beacon_cli(cli: Beacon) {
    let res = match cli.cmd {
        Command::Create(create) => create_beacon(create).await,
    };

    if let Err(e) = res {
        eprintln!("IPFS: {}", e);
    }
}

async fn create_beacon(args: Create) -> Result<(), Error> {
    let ipfs = IpfsClient::default();

    println!("Generating Keys...");

    let mut res = ipfs.key_list().await?;

    let videos = match search_keypairs(&args.videos_key, &mut res) {
        Some(kp) => kp.id,
        None => generate_key(&ipfs, &args.videos_key).await?,
    };

    let bans = match search_keypairs(&args.bans_key, &mut res) {
        Some(kp) => kp.id,
        None => generate_key(&ipfs, &args.bans_key).await?,
    };

    let mods = match search_keypairs(&args.mods_key, &mut res) {
        Some(kp) => kp.id,
        None => generate_key(&ipfs, &args.mods_key).await?,
    };

    update_video_list(&ipfs, &args.videos_key, &VideoList::default()).await?;
    update_bans_list(&ipfs, &args.bans_key, &Bans::default()).await?;
    update_mods_list(&ipfs, &args.mods_key, &Moderators::default()).await?;

    println!("Creating Beacon...");

    let mut config = get_config().await;

    config.chat.topic = args.chat_topic;
    config.chat.mods = mods.clone();
    config.chat.bans = bans.clone();

    config.video.pubsub_topic = args.video_topic;

    set_config(&config).await;

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

    println!("New Beacon CID => {}", &cid.to_string());

    Ok(())
}

pub fn search_keypairs(name: &str, res: &mut KeyListResponse) -> Option<KeyPair> {
    for (i, keypair) in res.keys.iter_mut().enumerate() {
        if keypair.name == name {
            return Some(res.keys.remove(i));
        }
    }

    None
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
