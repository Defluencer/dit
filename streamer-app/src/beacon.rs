use crate::utils::config::{get_config, set_config};
use crate::utils::dag_nodes::ipfs_dag_put_node_async;

use ipfs_api::response::{KeyListResponse, KeyPair};
use ipfs_api::IpfsClient;
use ipfs_api::KeyType;

use linked_data::beacon::{Beacon, Topics, VideoList};

/// Create beacon with key and topics passed in args.
pub async fn create_beacon(args: crate::Beacon) {
    let ipfs = IpfsClient::default();

    let res = match ipfs.key_list().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return;
        }
    };

    let (new_key, mut keypair) = match search_keypairs(&args.key_name, res) {
        Some(kp) => (false, kp),
        None => match ipfs
            .key_gen(
                &args.key_name,
                KeyType::Ed25519,
                64, /* Don't think this does anything... */
            )
            .await
        {
            Ok(res) => (true, res),
            Err(e) => {
                eprintln!("IPFS: {}", e);
                return;
            }
        },
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", keypair.name, keypair.id);

    if new_key {
        let cid = match ipfs_dag_put_node_async(&ipfs, &VideoList::default()).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("IPFS: {}", e);
                return;
            }
        };

        if let Err(e) = ipfs.pin_add(&cid.to_string(), false).await {
            eprintln!("IPFS: {}", e);
            return;
        }

        #[cfg(debug_assertions)]
        println!("IPFS: pin add => {}", &cid.to_string());

        println!("Publishing New IPNS Name...");

        if let Err(e) = ipfs
            .name_publish(&cid.to_string(), false, None, None, Some(&args.key_name))
            .await
        {
            eprintln!("IPFS: {}", e);
            return;
        }
    }

    let mut config = get_config().await;

    config.chat.pubsub_topic = args.chat_topic;
    config.video.pubsub_topic = args.video_topic;

    set_config(&config).await;

    let topics = Topics {
        live_video: config.video.pubsub_topic,
        live_chat: config.chat.pubsub_topic,
    };

    let peer_id = match ipfs.id(None).await {
        Ok(res) => res.id,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return;
        }
    };

    #[cfg(debug_assertions)]
    println!("IPFS: peer id => {}", &peer_id);

    keypair.id.insert_str(0, "/ipns/"); // add this in front to make a path

    let beacon = Beacon {
        topics,
        peer_id,
        video_list: keypair.id,
    };

    let cid = match ipfs_dag_put_node_async(&ipfs, &beacon).await {
        Ok(cid) => cid,
        Err(e) => {
            eprintln!("IPFS: {}", e);
            return;
        }
    };

    if let Err(e) = ipfs.pin_add(&cid.to_string(), false).await {
        eprintln!("IPFS: {}", e);
        return;
    }

    println!("Beacon CID => {}", &cid.to_string());
}

fn search_keypairs(name: &str, res: KeyListResponse) -> Option<KeyPair> {
    for keypair in res.keys {
        if keypair.name == name {
            return Some(keypair);
        }
    }

    None
}
