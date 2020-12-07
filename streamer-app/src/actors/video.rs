use crate::actors::archivist::Archive;
use crate::config::Track;
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
    // <StreamHash>/time/hour/0/minute/36/second/12/video/quality/1080p60/..
    #[serde(rename = "quality")]
    pub qualities: HashMap<String, IPLDLink>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/..
    #[serde(rename = "setup")]
    pub setup: IPLDLink,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/previous/..
    #[serde(rename = "previous")]
    pub previous: Option<IPLDLink>,
}

/// Codecs, qualities & initialization segments. Order matters!
#[derive(Serialize, Debug)]
pub struct SetupNode {
    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/quality
    #[serde(rename = "quality")]
    qualities: Vec<String>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/codec
    #[serde(rename = "codec")]
    codecs: Vec<String>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/initseg/0/..
    #[serde(rename = "initseg")]
    initialization_segments: Vec<IPLDLink>,
}

pub enum VideoData {
    Initialization(String, Bytes),
    Media(String, Bytes),
}

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
            match msg {
                VideoData::Initialization(quality, bytes) => self.init_seg(&quality, bytes).await,
                VideoData::Media(quality, bytes) => self.media_seg(&quality, bytes).await,
            }
        }
    }

    /// Process initialization segments.
    async fn init_seg(&mut self, quality: &str, data: Bytes) {
        //Panic on error because stream can't go on without setup node.

        let segment_cid = self
            .ipfs_add_async(data)
            .await
            .expect("SetupNode IPFS add failed");

        if !self.add_track(quality, segment_cid) {
            return;
        }

        self.sort_by_level();

        let setup_node_cid = ipfs_dag_put_node_async(&self.ipfs, &self.setup_node)
            .await
            .expect("SetupNode IPFS dag put failed");

        self.video_node.setup = IPLDLink {
            link: setup_node_cid,
        };
    }

    /// Add track info to setup node. Return true if all init are present
    fn add_track(&mut self, quality: &str, cid: Cid) -> bool {
        self.setup_node.qualities.push(quality.into());

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
    async fn media_seg(&mut self, quality: &str, data: Bytes) {
        let video_segment_cid = match self.ipfs_add_async(data).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("IPFS add failed {}", e);
                return;
            }
        };

        if !self.add_variant(quality, video_segment_cid) {
            return;
        }

        let video_node_cid = match ipfs_dag_put_node_async(&self.ipfs, &self.video_node).await {
            Ok(res) => res,
            Err(e) => {
                self.video_node.qualities.clear(); // re-set on error

                eprintln!("IPFS dag put failed {}", e);
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

    /// Add video data to IPFS. Return a CID.
    async fn ipfs_add_async(&mut self, data: Bytes) -> Result<Cid, Error> {
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

        let cid = Cid::try_from(response.hash).expect("Invalid Cid");

        #[cfg(debug_assertions)]
        println!("IPFS added => {}", &cid);

        Ok(cid)
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
            Ok(_) => println!("GossipSub published => {}", &cid),
            Err(e) => eprintln!("IPFS pubsub pub failed {}", e),
        }

        let link = IPLDLink { link: cid };

        self.video_node.previous = Some(link);
    }
}

/// Serialize then add dag node to IPFS. Return a CID.
async fn ipfs_dag_put_node_async<T>(ipfs: &IpfsClient, node: &T) -> Result<Cid, Error>
where
    T: ?Sized + Serialize,
{
    #[cfg(debug_assertions)]
    println!(
        "Serialize => {}",
        serde_json::to_string_pretty(node).unwrap()
    );

    let json_string = serde_json::to_string(node).expect("Serialize video node failed");

    let response = ipfs.dag_put(Cursor::new(json_string)).await?;

    let cid = Cid::try_from(response.cid.cid_string).expect("Invalid Cid");

    #[cfg(debug_assertions)]
    println!("Dag Put => {}", &cid);

    Ok(cid)
}
