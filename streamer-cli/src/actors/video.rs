use crate::actors::archivist::Archive;
use crate::utils::config::VideoConfig;
use crate::utils::dag_nodes::ipfs_dag_put_node_async;

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use ipfs_api::IpfsClient;

use linked_data::video::VideoNode;
use linked_data::IPLDLink;

use cid::Cid;

pub struct VideoAggregator {
    ipfs: IpfsClient,

    service_rx: UnboundedReceiver<VideoData>,
    archive_tx: Option<UnboundedSender<Archive>>,

    config: VideoConfig,

    track_len: usize,
    setup_link: Option<IPLDLink>,

    node_mint_count: usize,
    video_nodes: VecDeque<VideoNode>,

    previous: Option<IPLDLink>,
}

#[derive(Debug)]
pub enum VideoData {
    Segment((PathBuf, Cid)),
    Setup((IPLDLink, usize)),
}

impl VideoAggregator {
    pub fn new(
        ipfs: IpfsClient,
        service_rx: UnboundedReceiver<VideoData>,
        archive_tx: Option<UnboundedSender<Archive>>,
        config: VideoConfig,
    ) -> Self {
        Self {
            ipfs,

            service_rx,
            archive_tx,

            config,

            track_len: 0,
            setup_link: None,

            node_mint_count: 0,
            video_nodes: VecDeque::with_capacity(5),
            previous: None,
        }
    }

    pub async fn start(&mut self) {
        println!("✅ Video System Online");

        while let Some(msg) = self.service_rx.recv().await {
            match msg {
                VideoData::Segment((path, cid)) => self.media_seg(path, cid).await,
                VideoData::Setup((link, len)) => {
                    self.track_len = len;
                    self.setup_link = Some(link);
                }
            }
        }

        println!("❌ Video System Offline");
    }

    /// Update or create VideoNode in queue then try to mint one.
    async fn media_seg(&mut self, path: PathBuf, cid: Cid) {
        let quality = path
            .parent()
            .expect("Orphan path!")
            .file_name()
            .expect("Dir with no name!")
            .to_str()
            .expect("Invalid Unicode");

        //absolute index from ffmpeg
        let index = path
            .file_stem()
            .expect("Not file stem")
            .to_str()
            .expect("Invalid Unicode")
            .parse::<usize>()
            .expect("Not a number");

        // relative index for in memory video nodes
        let buffer_index = index - self.node_mint_count;

        if let Some(node) = self.video_nodes.get_mut(buffer_index) {
            node.tracks.insert(quality.to_owned(), cid.into());

            node.setup = self.setup_link;

            // Set previous field only for the next node to be minted
            if buffer_index == 0 {
                node.previous = self.previous;
            }
        } else {
            let mut tracks = HashMap::with_capacity(4);

            tracks.insert(quality.to_owned(), cid.into());

            let setup = self.setup_link;

            let previous = None;

            let node = VideoNode {
                tracks,
                setup,
                previous,
            };

            self.video_nodes.push_back(node);
        }

        // try to mint in case something failed previously
        while let Some(cid) = self.mint_video_node().await {
            if let Some(archive_tx) = self.archive_tx.as_ref() {
                let msg = Archive::Video(cid);

                if let Err(error) = archive_tx.send(msg) {
                    eprintln!("❗ Archive receiver hung up! Error: {}", error);
                }
            }

            if self.config.pubsub_enable {
                let topic = &self.config.pubsub_topic;

                if let Err(e) = self.ipfs.pubsub_pub(topic, &cid.to_string()).await {
                    eprintln!("❗ IPFS: pubsub pub failed {}", e);
                }
            }
        }

        #[cfg(debug_assertions)]
        println!("Video: {} buffered nodes", self.video_nodes.len());
    }

    /// Mint the first VideoNode in queue if it meets all requirements.
    async fn mint_video_node(&mut self) -> Option<Cid> {
        let node = self.video_nodes.front_mut()?;

        node.setup = self.setup_link;

        node.setup?;

        if node.tracks.len() != self.track_len {
            return None;
        }

        if node.previous.is_none() && self.node_mint_count > 0 {
            return None;
        }

        let cid = match ipfs_dag_put_node_async(&self.ipfs, node).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("❗ IPFS: dag put failed {}", e);
                return None;
            }
        };

        self.video_nodes.pop_front();
        self.node_mint_count += 1;
        self.previous = Some(cid.into());

        println!("Video Node Minted => {}", &cid.to_string());

        Some(cid)
    }
}
