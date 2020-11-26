use crate::actors::archivist::Archive;
use crate::dag_nodes::IPLDLink;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Cursor;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use hyper::body::Bytes;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::Serialize;

use cid::Cid;

/// Links all variants, allowing selection of video quality. Also link to the previous video node.
#[derive(Serialize, Debug)]
pub struct VideoNode {
    // <StreamHash>/time/hour/0/minute/36/second/12/video/init/1080p60/..
    #[serde(rename = "init")]
    pub initialization_segments: HashMap<String, IPLDLink>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/quality/1080p60/..
    #[serde(rename = "quality")]
    pub qualities: HashMap<String, IPLDLink>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/previous/..
    pub previous: Option<IPLDLink>,
}

pub struct VideoAggregator {
    ipfs: IpfsClient,

    archive_tx: Sender<Archive>,

    video_rx: Receiver<(String, Bytes, bool)>,

    gossipsub_topic: String,

    video_node: VideoNode,

    qualities: usize,
}

impl VideoAggregator {
    pub fn new(
        ipfs: IpfsClient,
        video_rx: Receiver<(String, Bytes, bool)>,
        archive_tx: Sender<Archive>,
        gossipsub_topic: String,
        variants: usize,
    ) -> Self {
        Self {
            ipfs,

            archive_tx,
            video_rx,

            gossipsub_topic,

            video_node: VideoNode {
                initialization_segments: HashMap::with_capacity(variants),
                qualities: HashMap::with_capacity(variants),
                previous: None,
            },

            qualities: variants,
        }
    }

    pub async fn aggregate(&mut self) {
        println!("Video Online...");

        while let Some((variant, data, init)) = self.video_rx.recv().await {
            let video_segment_cid = match self.add_video(data).await {
                Ok(cid) => cid,
                Err(e) => {
                    eprintln!("add_video failed {}", e);
                    continue;
                }
            };

            if !self.add_variant(variant, video_segment_cid, init) {
                continue;
            }

            let video_node_cid = match self.collect_variants().await {
                Ok(res) => res,
                Err(e) => {
                    self.video_node.qualities.clear(); //reset the node

                    eprintln!("collect_variants failed {}", e);

                    continue;
                }
            };

            let msg = Archive::Video(video_node_cid);

            if let Err(error) = self.archive_tx.send(msg).await {
                eprintln!("Archive receiver hung up {}", error);
            }

            self.publish(video_node_cid).await;
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

        let cid = Cid::try_from(response.hash).expect("add_video failed");

        #[cfg(debug_assertions)]
        println!("IPFS added => {}", &cid);

        Ok(cid)
    }

    /// Add CID to stream variants dag node. Return true if all stream variants were added.
    fn add_variant(&mut self, variant: String, cid: Cid, init: bool) -> bool {
        let link = IPLDLink { link: cid };

        if init {
            self.video_node
                .initialization_segments
                .insert(variant, link);

            false
        } else {
            self.video_node.qualities.insert(variant, link);

            self.video_node.qualities.len() >= self.qualities
        }
    }

    /// Add stream variants dag node to IPFS. Return a CID.
    async fn collect_variants(&mut self) -> Result<Cid, Error> {
        let node = &self.video_node;

        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(node).unwrap());

        let json_string = serde_json::to_string(node).expect("collect_variants failed");

        let response = self.ipfs.dag_put(Cursor::new(json_string)).await?;

        let cid = Cid::try_from(response.cid.cid_string).expect("collect_variants failed");

        self.video_node.qualities.clear();

        Ok(cid)
    }

    /// Publish video node CID to configured topic using GossipSub.
    async fn publish(&mut self, cid: Cid) {
        let topic = &self.gossipsub_topic;

        match self.ipfs.pubsub_pub(topic, &cid.to_string()).await {
            Ok(_) => println!("GossipSub published => {}", &cid),
            Err(e) => eprintln!("IPFS pubsub pub failed {}", e),
        }

        let link = IPLDLink { link: cid };

        self.video_node.previous = Some(link);
    }
}
