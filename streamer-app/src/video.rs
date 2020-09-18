use crate::chronicler::Archive;
use crate::config::Config;
use crate::dag_nodes::{IPLDLink, LiveNode, VariantsNode};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Cursor;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use hyper::body::Bytes;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use cid::Cid;

pub struct VideoAggregator {
    ipfs: IpfsClient,

    archive_tx: Sender<Archive>,
    video_rx: Receiver<(String, Bytes)>,

    config: Config,

    node: VariantsNode,
    previous_link: Option<IPLDLink>,
}

impl VideoAggregator {
    pub fn new(
        ipfs: IpfsClient,
        video_rx: Receiver<(String, Bytes)>,
        archive_tx: Sender<Archive>,
        config: Config,
    ) -> Self {
        Self {
            ipfs,
            archive_tx,
            video_rx,
            config,

            node: VariantsNode {
                variants: HashMap::with_capacity(4),
            },

            previous_link: None,
        }
    }

    pub async fn aggregate(&mut self) {
        while let Some((variant, data)) = self.video_rx.recv().await {
            let video_segment_cid = match self.add_video(data).await {
                Ok(cid) => cid,
                Err(e) => {
                    eprintln!("IPFS add failed {}", e);
                    continue;
                }
            };

            let should_collect = self.add_variant(variant, video_segment_cid);

            if !should_collect {
                continue;
            }

            let variants_node_cid = match self.collect_variants().await {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("IPFS dag put failed {}", e);
                    continue;
                }
            };

            let msg = Archive::Video(variants_node_cid.clone());

            if let Err(error) = self.archive_tx.send(msg).await {
                eprintln!("Archive receiver hung up {}", error);
            }

            let live_node_cid = match self.add_live(variants_node_cid).await {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("IPFS dag put failed {}", e);
                    continue;
                }
            };

            self.publish(live_node_cid).await;
        }
    }

    /// Add video data to IPFS. Return a CID.
    async fn add_video(&mut self, data: Bytes) -> Result<Cid, Error> {
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

        let response = self.ipfs.add_with_options(Cursor::new(data), add).await?;

        let cid = Cid::try_from(response.hash).expect("CID from dag put response failed");

        #[cfg(debug_assertions)]
        println!("IPFS added => {}", &cid);

        Ok(cid)
    }

    /// Add CID to stream variants dag node. Return true if all stream variants were added.
    fn add_variant(&mut self, variant: String, cid: Cid) -> bool {
        let link = IPLDLink { link: cid };

        self.node.variants.insert(variant, link);

        self.node.variants.len() >= self.config.variants
    }

    /// Add stream variants dag node to IPFS. Return a CID.
    async fn collect_variants(&mut self) -> Result<Cid, Error> {
        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(&self.node).unwrap());

        let json_string = serde_json::to_string(&self.node).expect("Can't serialize variants node");

        let response = self.ipfs.dag_put(Cursor::new(json_string)).await?;

        let cid = Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed");

        self.node.variants.clear();

        Ok(cid)
    }

    /// Add live dag node to IPFS. Return a CID.
    async fn add_live(&mut self, cid: Cid) -> Result<Cid, Error> {
        let live_node = LiveNode {
            current: IPLDLink { link: cid },
            previous: self.previous_link.clone(),
        };

        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(&live_node).unwrap());

        let json_string = serde_json::to_string(&live_node).expect("Can't serialize live node");

        let response = self.ipfs.dag_put(Cursor::new(json_string)).await?;

        let cid = Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed");

        Ok(cid)
    }

    /// Publish live dag node CID to configured topic using GossipSub.
    async fn publish(&mut self, cid: Cid) {
        let topic = &self.config.gossipsub_topics.video;

        match self.ipfs.pubsub_pub(topic, &cid.to_string()).await {
            Ok(_) => println!("GossipSub published => {}", &cid),
            Err(e) => eprintln!("IPFS pubsub pub failed {}", e),
        }

        let link = IPLDLink { link: cid };

        self.previous_link = Some(link);
    }
}
