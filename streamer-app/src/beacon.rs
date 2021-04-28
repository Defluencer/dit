use crate::utils::config::{get_config, set_config};
use crate::utils::dag_nodes::ipfs_dag_put_node_async;
use crate::video::update_video_list;
use crate::DEFAULT_KEY;

use ipfs_api::response::{Error, KeyListResponse, KeyPair};
use ipfs_api::IpfsClient;
use ipfs_api::KeyType;

use linked_data::beacon::{Topics, VideoList};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Beacon {
    /// IPNS key name for video list resolution.
    #[structopt(short, long, default_value = DEFAULT_KEY)]
    key_name: String,

    /// GossipSub topic for receiving chat messages.
    #[structopt(short, long)]
    chat_topic: String,

    /// GossipSub topic for video broadcasting.
    #[structopt(short, long)]
    video_topic: String,
}

pub async fn beacon_cli(args: Beacon) {
    let ipfs = IpfsClient::default();

    if let Err(e) = create_beacon(&ipfs, args).await {
        eprintln!("IPFS: {}", e);
    }
}

async fn create_beacon(ipfs: &IpfsClient, args: Beacon) -> Result<(), Error> {
    let res = ipfs.key_list().await?;

    let (new_key, mut keypair) = match search_keypairs(&args.key_name, res) {
        Some(kp) => (false, kp),
        None => {
            println!("Generating Key...");

            let res = ipfs
                .key_gen(
                    &args.key_name,
                    KeyType::Ed25519,
                    64, /* Don't think this does anything... */
                )
                .await?;

            (true, res)
        }
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", keypair.name, keypair.id);

    if new_key {
        update_video_list(&ipfs, &args.key_name, &VideoList::default()).await?;
    }

    println!("Creating Beacon...");

    let mut config = get_config().await;

    config.chat.pubsub_topic = args.chat_topic;
    config.video.pubsub_topic = args.video_topic;

    set_config(&config).await;

    let topics = Topics {
        live_video: config.video.pubsub_topic,
        live_chat: config.chat.pubsub_topic,
    };

    let res = ipfs.id(None).await?;
    let peer_id = res.id;

    #[cfg(debug_assertions)]
    println!("IPFS: peer id => {}", &peer_id);

    keypair.id.insert_str(0, "/ipns/"); // add this in front to make a path

    let beacon = linked_data::beacon::Beacon {
        topics,
        peer_id,
        video_list: keypair.id,
        //TODO chat & mederation
        //chat_allow: None,
        //chat_block: None,
        //chat_mods: None,
    };

    let cid = ipfs_dag_put_node_async(&ipfs, &beacon).await?;

    ipfs.pin_add(&cid.to_string(), true).await?;

    println!("Beacon CID => {}", &cid.to_string());

    Ok(())
}

pub fn search_keypairs(name: &str, res: KeyListResponse) -> Option<KeyPair> {
    for keypair in res.keys {
        if keypair.name == name {
            return Some(keypair);
        }
    }

    None
}
