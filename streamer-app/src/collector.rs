use crate::config::Config;
use crate::hash_timecode::IPLDLink;
use crate::hash_timecode::Timecode;
use crate::stream_links::{LiveNode, VariantsNode};

use std::collections::HashMap;
use std::io::Cursor;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use hyper::body::Bytes;

use ipfs_api::IpfsClient;

pub async fn collect_video_data(
    ipfs: IpfsClient,
    mut timecode_tx: Sender<Timecode>,
    mut rx: Receiver<(String, Bytes)>,
    config: &Config,
) {
    let mut node = VariantsNode {
        variants: HashMap::with_capacity(4),
    };

    let mut previous_link: Option<IPLDLink> = None;

    while let Some((variant, data)) = rx.recv().await {
        let add = ipfs_api::request::Add {
            trickle: None,
            only_hash: None,
            wrap_with_directory: None,
            chunker: None,
            pin: Some(false),
            raw_leaves: None,
            cid_version: Some(1),
            hash: None,
            inline: None,
            inline_limit: None,
        };

        let video_segment_cid = match ipfs.add_with_options(Cursor::new(data), add).await {
            Ok(result) => result.hash,
            Err(e) => {
                eprintln!("IPFS add failed {}", e);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!("IPFS add => {}", &video_segment_cid);

        let link = IPLDLink {
            link: video_segment_cid,
        };

        node.variants.insert(variant, link);

        if node.variants.len() < 4 {
            //TODO smart numbering of variant => insert new until key already exist then count number of keys
            //TODO replace 4 with number of variant stream
            //TODO make sure the 4 variants are synced
            continue;
        }

        #[cfg(debug_assertions)]
        println!("{:#?}", node);

        let json_string = serde_json::to_string(&node).expect("Can't serialize variants node");

        let variants_node_cid = match ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                continue;
            }
        };

        node.variants.clear();

        let msg = Timecode::Add(variants_node_cid.clone());

        if let Err(error) = timecode_tx.send(msg).await {
            eprintln!("Timecode receiver hung up {}", error);
        }

        let live_node = LiveNode {
            current: IPLDLink {
                link: variants_node_cid,
            },
            previous: previous_link.clone(),
        };

        let json_string = serde_json::to_string(&live_node).expect("Can't serialize live node");

        let live_node_cid = match ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                continue;
            }
        };

        #[cfg(debug_assertions)]
        println!("Live node CID => {}", &live_node_cid);

        let link = IPLDLink {
            link: live_node_cid.clone(),
        };

        previous_link = Some(link);

        match ipfs
            .pubsub_pub(&config.gossipsub_topic, &live_node_cid)
            .await
        {
            Ok(_) => {
                println!("GossipSub publish => {}", &live_node_cid);
            }
            Err(e) => {
                eprintln!("IPFS pubsub pub failed {}", e);
            }
        }
    }
}
