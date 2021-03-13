use crate::actors::archivist::Archive;
use crate::server::{FMP4, MP4};
use crate::utils::ipfs_dag_put_node_async;

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use ipfs_api::IpfsClient;

use linked_data::config::VideoConfig;
use linked_data::video::{SetupNode, VideoNode};
use linked_data::IPLDLink;

use cid::Cid;

use m3u8_rs::playlist::MasterPlaylist;

#[derive(Debug)]
pub enum VideoData {
    Playlist(MasterPlaylist),
    Segment((PathBuf, Cid)),
}

pub struct VideoAggregator {
    ipfs: IpfsClient,

    archive_tx: Option<UnboundedSender<Archive>>,

    video_rx: UnboundedReceiver<VideoData>,

    config: VideoConfig,

    init_map: HashMap<String, Cid>,
    setup_count: usize,
    setup_link: Option<IPLDLink>,
    setup_node: Option<SetupNode>,

    node_mint_count: usize,
    video_nodes: VecDeque<VideoNode>,
    previous: Option<IPLDLink>,
}

impl VideoAggregator {
    pub fn new(
        ipfs: IpfsClient,
        video_rx: UnboundedReceiver<VideoData>,
        archive_tx: Option<UnboundedSender<Archive>>,
        config: VideoConfig,
    ) -> Self {
        Self {
            ipfs,

            archive_tx,
            video_rx,

            config,

            init_map: HashMap::with_capacity(4),
            setup_count: 0,
            setup_link: None,
            setup_node: None,

            node_mint_count: 0,
            video_nodes: VecDeque::with_capacity(5),
            previous: None,
        }
    }

    pub async fn start(&mut self) {
        println!("Video System Online");

        while let Some(msg) = self.video_rx.recv().await {
            match msg {
                VideoData::Playlist(pl) => self.process_master_playlist(pl).await,
                VideoData::Segment((path, cid)) => self.split_segments(path, cid).await,
            }
        }

        println!("Video System Offline");
    }

    async fn split_segments(&mut self, path: PathBuf, cid: Cid) {
        if path.extension().unwrap() == FMP4 {
            return self.media_seg(path, cid).await;
        }

        if path.extension().unwrap() == MP4 {
            return self.init_seg(path, cid).await;
        }
    }

    /// Create or update SetupNode based on master playlist then try to mint it.
    async fn process_master_playlist(&mut self, pl: MasterPlaylist) {
        #[cfg(debug_assertions)]
        println!("{:#?}", pl);

        self.setup_count = pl.variants.len();

        let initialization_segments = Vec::with_capacity(self.setup_count);
        let mut qualities = Vec::with_capacity(self.setup_count);
        let mut codecs = Vec::with_capacity(self.setup_count);
        let mut bandwidths = Vec::with_capacity(self.setup_count);

        //TODO reorder vectors based on bandwidth would fix ordering constraint we have now.

        //Assumes variants ordering is highest to lowest quality.
        for variant in pl.variants.into_iter().rev() {
            match variant.codecs {
                Some(codec) => codecs.push(format!(r#"video/mp4; codecs="{}"#, codec)),
                None => codecs.push(String::new()),
            }

            match variant.bandwidth.parse::<usize>() {
                Ok(bw) => bandwidths.push(bw),
                Err(_) => bandwidths.push(0),
            }

            let path = PathBuf::from(variant.uri);

            let quality = path
                .parent()
                .expect("Orphan path!")
                .file_name()
                .expect("Dir with no name!")
                .to_str()
                .expect("Invalid Unicode");

            qualities.push(quality.to_owned());
        }

        let setup_node = SetupNode {
            initialization_segments,
            qualities,
            codecs,
            bandwidths,
        };

        self.setup_node = Some(setup_node);

        self.try_mint_setup_node().await;
    }

    /// Mint SetupNode if it meets all requirements.
    async fn try_mint_setup_node(&mut self) {
        if self.init_map.is_empty() {
            return;
        }

        if self.setup_node.is_none() {
            return;
        }

        if self.init_map.len() != self.setup_count {
            return;
        }

        let setup_node = self.setup_node.as_mut().unwrap();

        for quality in setup_node.qualities.iter() {
            let cid = self.init_map[quality];

            let link = IPLDLink { link: cid };

            setup_node.initialization_segments.push(link);
        }

        // Panic because can't be recovered from anyway
        let cid = ipfs_dag_put_node_async(&self.ipfs, setup_node)
            .await
            .expect("IPFS: SetupNode dag put failed");

        println!("Setup Node Minted => {}", &cid.to_string());

        self.setup_link = Some(IPLDLink { link: cid });
        self.setup_node = None;
        self.init_map = HashMap::with_capacity(0);
    }

    /// Update map of quality to cid for initialization segments then try to mint SetupNode.
    async fn init_seg(&mut self, path: PathBuf, cid: Cid) {
        let quality = path
            .parent()
            .expect("Orphan path!")
            .file_name()
            .expect("Dir with no name!")
            .to_str()
            .expect("Invalid Unicode");

        self.init_map.insert(quality.to_owned(), cid);

        self.try_mint_setup_node().await;
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
            node.qualities
                .insert(quality.to_owned(), IPLDLink { link: cid });

            node.setup = self.setup_link;

            // Set previous field only for the next node to be minted
            if buffer_index == 0 {
                node.previous = self.previous;
            }
        } else {
            let mut qualities = HashMap::with_capacity(4);

            qualities.insert(quality.to_owned(), IPLDLink { link: cid });

            let setup = self.setup_link;

            let previous = None;

            let node = VideoNode {
                qualities,
                setup,
                previous,
            };

            self.video_nodes.push_back(node);
        }

        while let Some(cid) = self.mint_video_node().await {
            if let Some(archive_tx) = self.archive_tx.as_ref() {
                let msg = Archive::Video(cid);
                if let Err(error) = archive_tx.send(msg) {
                    eprintln!("Archive receiver hung up! Error: {}", error);
                }
            }

            if self.config.pubsub_enable {
                let topic = &self.config.pubsub_topic;

                if let Err(e) = self.ipfs.pubsub_pub(topic, &cid.to_string()).await {
                    eprintln!("IPFS: pubsub pub failed {}", e);
                }
            }
        }

        #[cfg(debug_assertions)]
        println!("Video: {} buffered nodes", self.video_nodes.len());
    }

    /// Mint the first VideoNode in queue if it meets all requirements.
    async fn mint_video_node(&mut self) -> Option<Cid> {
        let node = self.video_nodes.get_mut(0)?;

        node.setup = self.setup_link;

        node.setup?;

        if node.qualities.len() != self.setup_count {
            return None;
        }

        if node.previous.is_none() && self.node_mint_count > 0 {
            return None;
        }

        let cid = match ipfs_dag_put_node_async(&self.ipfs, node).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("IPFS: dag put failed {}", e);
                return None;
            }
        };

        self.video_nodes.pop_front();
        self.node_mint_count += 1;
        self.previous = Some(IPLDLink { link: cid });

        println!("Video Node Minted => {}", &cid.to_string());

        Some(cid)
    }
}
