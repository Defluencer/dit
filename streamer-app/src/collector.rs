use crate::config::Config;
use crate::hash_timecode::IPLDLink;
use crate::hash_timecode::Timecode;

use std::collections::HashMap;
use std::io::Cursor;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use hyper::body::Bytes;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::Serialize;

pub struct HashVideo {
    ipfs: IpfsClient,

    timecode_tx: Sender<Timecode>,
    video_rx: Receiver<(String, Bytes)>,

    config: Config,

    node: VariantsNode,
    previous_link: Option<IPLDLink>,
}

impl HashVideo {
    pub fn new(
        ipfs: IpfsClient,
        video_rx: Receiver<(String, Bytes)>,
        timecode_tx: Sender<Timecode>,
        config: Config,
    ) -> Self {
        Self {
            ipfs,
            timecode_tx,
            video_rx,
            config,

            node: VariantsNode {
                variants: HashMap::with_capacity(4),
            },

            previous_link: None,
        }
    }

    pub async fn collect(&mut self) {
        while let Some((variant, data)) = self.video_rx.recv().await {
            let cid = match self.add_video(data).await {
                Ok(cid) => cid,
                Err(e) => {
                    eprintln!("IPFS add failed {}", e);
                    continue;
                }
            };

            let should_collect = self.add_variant(variant, cid);

            #[cfg(debug_assertions)]
            println!("{:#?}", self.node);

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

            let msg = Timecode::Add(variants_node_cid.clone());

            if let Err(error) = self.timecode_tx.send(msg).await {
                eprintln!("Timecode receiver hung up {}", error);
            }

            let live_node_cid = match self.add_live(variants_node_cid).await {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("IPFS dag put failed {}", e);
                    continue;
                }
            };

            #[cfg(debug_assertions)]
            println!("Live node CID => {}", &live_node_cid);

            self.publish(live_node_cid).await;
        }
    }

    /// Add video data to IPFS. Return a CID.
    async fn add_video(&mut self, data: Bytes) -> Result<String, Error> {
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

        let video_segment_cid = response.hash;

        #[cfg(debug_assertions)]
        println!("IPFS add => {}", &video_segment_cid);

        Ok(video_segment_cid)
    }

    /// Add CID to stream variants dag node. Return true if all stream variants were added.
    fn add_variant(&mut self, variant: String, cid: String) -> bool {
        let link = IPLDLink { link: cid };

        self.node.variants.insert(variant, link);

        //TODO smart numbering of variant => insert new until key already exist then count number of keys
        //TODO replace 4 with number of variant stream
        //TODO make sure the 4 variants are synced

        self.node.variants.len() >= 4
    }

    /// Add stream variants dag node to IPFS. Return a CID.
    async fn collect_variants(&mut self) -> Result<String, Error> {
        let json_string = serde_json::to_string(&self.node).expect("Can't serialize variants node");

        let response = self.ipfs.dag_put(Cursor::new(json_string)).await?;

        self.node.variants.clear();

        Ok(response.cid.cid_string)
    }

    /// Add live dag node to IPFS. Return a CID.
    async fn add_live(&mut self, cid: String) -> Result<String, Error> {
        let live_node = LiveNode {
            current: IPLDLink { link: cid },
            previous: self.previous_link.clone(),
        };

        let json_string = serde_json::to_string(&live_node).expect("Can't serialize live node");

        let response = self.ipfs.dag_put(Cursor::new(json_string)).await?;

        Ok(response.cid.cid_string)
    }

    /// Publish live dag node CID to configured topic using GossipSub.
    async fn publish(&mut self, cid: String) {
        match self
            .ipfs
            .pubsub_pub(&self.config.gossipsub_topic, &cid)
            .await
        {
            Ok(_) => {
                println!("GossipSub publish => {}", &cid);
            }
            Err(e) => {
                eprintln!("IPFS pubsub pub failed {}", e);
            }
        }

        let link = IPLDLink { link: cid };

        self.previous_link = Some(link);
    }
}

/// Link the current stream variants dag node and the previous live dag node.
/// Allow rewind/buffer previous video segments.
#[derive(Serialize, Debug)]
pub struct LiveNode {
    pub current: IPLDLink,
    pub previous: Option<IPLDLink>,
}

// egg ../<StreamHash>/timecode/hours/0/minutes/36/seconds/12/variants/1080p60/.. => video blocks

/// Link all stream variants.
/// Allow viewer to select video quality.
#[derive(Serialize, Debug)]
pub struct VariantsNode {
    pub variants: HashMap<String, IPLDLink>,
}
