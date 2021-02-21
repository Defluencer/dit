use crate::actors::archivist::Archive;
use crate::server::{FMP4, MP4};
use crate::utils::ipfs_dag_put_node_async;

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use ipfs_api::IpfsClient;

use linked_data::config::Track;
use linked_data::video::{SetupNode, VideoNode};
use linked_data::IPLDLink;

use cid::Cid;

pub type VideoData = (PathBuf, Cid);

pub struct VideoAggregator {
    ipfs: IpfsClient,

    archive_tx: Sender<Archive>,

    video_rx: Receiver<VideoData>,

    gossipsub_topic: String,

    setup_node: SetupNode,
    video_node: VideoNode,

    tracks: HashMap<String, Track>,
}

impl VideoAggregator {
    pub fn new(
        ipfs: IpfsClient,
        video_rx: Receiver<VideoData>,
        archive_tx: Sender<Archive>,
        gossipsub_topic: String,
        tracks: HashMap<String, Track>,
    ) -> Self {
        Self {
            ipfs,

            archive_tx,
            video_rx,

            gossipsub_topic,

            setup_node: SetupNode {
                codecs: Vec::with_capacity(tracks.len()),
                qualities: Vec::with_capacity(tracks.len()),
                initialization_segments: Vec::with_capacity(tracks.len()),
            },

            video_node: VideoNode {
                qualities: HashMap::with_capacity(tracks.len()),
                setup: IPLDLink::default(),
                previous: None,
            },

            tracks,
        }
    }

    pub async fn start_receiving(&mut self) {
        println!("Video System Online");

        while let Some(msg) = self.video_rx.recv().await {
            let (path, cid) = msg;

            #[cfg(debug_assertions)]
            println!("IPFS: add => {}", &cid.to_string());

            let quality = path
                .parent()
                .expect("Orphan path!")
                .file_name()
                .expect("Dir with no name!")
                .to_str()
                .expect("Invalid Unicode!");

            if path.extension().unwrap() == FMP4 {
                self.media_seg(quality, cid).await;
                continue;
            }

            if path.extension().unwrap() == MP4 {
                self.init_seg(quality, cid).await;
                continue;
            }
        }
    }

    /// Process initialization segments.
    async fn init_seg(&mut self, quality: &str, cid: Cid) {
        if !self.add_track(quality, cid) {
            return;
        }

        self.sort_by_level();

        //Panic on error because live stream can't go on without setup node anyway.
        let setup_node_cid = ipfs_dag_put_node_async(&self.ipfs, &self.setup_node)
            .await
            .expect("IPFS: dag put failed");

        self.video_node.setup = IPLDLink {
            link: setup_node_cid,
        };
    }

    /// Add track info to setup node. Return true if all init are present
    fn add_track(&mut self, quality: &str, cid: Cid) -> bool {
        self.setup_node.qualities.push(quality.into());

        //TODO handle case if tracks does not contain quality
        let codec = self.tracks[quality].codec.clone();

        self.setup_node.codecs.push(codec);

        let link = IPLDLink { link: cid };

        self.setup_node.initialization_segments.push(link);

        self.setup_node.initialization_segments.len() >= self.tracks.len()
    }

    /// Sort setup node vectors by track level.
    fn sort_by_level(&mut self) {
        for i in 0..self.tracks.len() {
            loop {
                let level = self.tracks[&self.setup_node.qualities[i]].level;

                if i == level {
                    break;
                }

                self.setup_node.qualities.swap(i, level);
                self.setup_node.codecs.swap(i, level);
                self.setup_node.initialization_segments.swap(i, level);
            }
        }
    }

    /// Process media segments.
    async fn media_seg(&mut self, quality: &str, cid: Cid) {
        if !self.add_variant(quality, cid) {
            return;
        }

        let video_node_cid = match ipfs_dag_put_node_async(&self.ipfs, &self.video_node).await {
            Ok(res) => res,
            Err(e) => {
                self.video_node.qualities.clear(); // reset on error

                eprintln!("IPFS: dag put failed {}", e);
                return;
            }
        };

        self.video_node.qualities.clear();

        let msg = Archive::Video(video_node_cid);

        if let Err(error) = self.archive_tx.send(msg).await {
            eprintln!("Archive receiver hung up! Error: {}", error);
        }

        self.publish(video_node_cid).await;
    }

    /// Add CID to video dag node. Return true if all variants were added.
    fn add_variant(&mut self, quality: &str, cid: Cid) -> bool {
        let link = IPLDLink { link: cid };

        self.video_node.qualities.insert(quality.to_string(), link);

        self.video_node.qualities.len() >= self.tracks.len()
    }

    /// Publish video node CID to configured topic using GossipSub.
    async fn publish(&mut self, cid: Cid) {
        let topic = &self.gossipsub_topic;

        match self.ipfs.pubsub_pub(topic, &cid.to_string()).await {
            Ok(_) => println!("IPFS: GossipSub published => {}", &cid),
            Err(e) => eprintln!("IPFS: pubsub pub failed {}", e),
        }

        let link = IPLDLink { link: cid };

        self.video_node.previous = Some(link);
    }
}
