use crate::actors::archivist::Archive;
use crate::server::{FMP4, MP4};
use crate::utils::ipfs_dag_put_node_async;

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use ipfs_api::IpfsClient;

use linked_data::video::{SetupNode, VideoNode};
use linked_data::IPLDLink;

use cid::Cid;

use m3u8_rs::playlist::MasterPlaylist;
use m3u8_rs::playlist::Playlist;

#[derive(Debug)]
pub enum VideoData {
    Playlist(Playlist),
    Segment((PathBuf, Cid)),
}

pub struct VideoAggregator {
    ipfs: IpfsClient,

    archive_tx: Sender<Archive>,

    video_rx: Receiver<VideoData>,

    topic: String,

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
        video_rx: Receiver<VideoData>,
        archive_tx: Sender<Archive>,
        topic: String,
    ) -> Self {
        Self {
            ipfs,

            archive_tx,
            video_rx,

            topic,

            init_map: HashMap::with_capacity(4),
            setup_count: 0,
            setup_link: None,
            setup_node: None,

            node_mint_count: 0,
            video_nodes: VecDeque::with_capacity(5),
            previous: None,
        }
    }

    pub async fn start_receiving(&mut self) {
        println!("Video System Online");

        while let Some(msg) = self.video_rx.recv().await {
            match msg {
                VideoData::Playlist(pl) => {
                    if let Playlist::MasterPlaylist(pl) = pl {
                        self.process_master_playlist(pl).await
                    }
                }
                VideoData::Segment((path, cid)) => self.split_segments(path, cid).await,
            }
        }
    }

    async fn process_master_playlist(&mut self, pl: MasterPlaylist) {
        #[cfg(debug_assertions)]
        println!("{:#?}", pl);

        self.setup_count = pl.variants.len();

        let initialization_segments = Vec::with_capacity(self.setup_count);
        let mut qualities = Vec::with_capacity(self.setup_count);
        let mut codecs = Vec::with_capacity(self.setup_count);
        let mut bandwidths = Vec::with_capacity(self.setup_count);

        //TODO reorder vectors based on bandwidth
        //would fix ordering constraint we have now

        //Assumes variants ordering is highest to lowest quality
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

            /* if let Some(cid) = self.init_map.remove(quality) {
                initialization_segments.push(IPLDLink { link: cid })
            }; */

            qualities.push(quality.to_owned());
        }

        let setup_node = SetupNode {
            initialization_segments,
            qualities,
            codecs,
            bandwidths,
        };

        self.setup_node = Some(setup_node);

        self.mint_setup_node().await;
    }

    async fn mint_setup_node(&mut self) {
        if self.init_map.is_empty() {
            return;
        }

        let setup_node = match self.setup_node.as_mut() {
            Some(sn) => sn,
            None => return,
        };

        if self.init_map.len() != self.setup_count {
            return;
        }

        for quality in setup_node.qualities.iter() {
            let cid = self.init_map[quality];

            let link = IPLDLink { link: cid };

            setup_node.initialization_segments.push(link);
        }

        //Panic on error because live stream can't go on without setup node anyway.
        let cid = ipfs_dag_put_node_async(&self.ipfs, setup_node)
            .await
            .expect("SetupNode Dag Put Failed");

        self.setup_link = Some(IPLDLink { link: cid });
        self.setup_node = None;
        self.init_map = HashMap::with_capacity(0);
    }

    async fn split_segments(&mut self, path: PathBuf, cid: Cid) {
        #[cfg(debug_assertions)]
        println!("IPFS: add => {}", &cid.to_string());

        if path.extension().unwrap() == FMP4 {
            return self.media_seg(path, cid).await;
        }

        if path.extension().unwrap() == MP4 {
            return self.init_seg(path, cid).await;
        }
    }

    /// Process initialization segments.
    async fn init_seg(&mut self, path: PathBuf, cid: Cid) {
        let quality = path
            .parent()
            .expect("Orphan path!")
            .file_name()
            .expect("Dir with no name!")
            .to_str()
            .expect("Invalid Unicode");

        self.init_map.insert(quality.to_owned(), cid);

        self.mint_setup_node().await;
    }

    /// Process media segments.
    async fn media_seg(&mut self, path: PathBuf, cid: Cid) {
        let quality = path
            .parent()
            .expect("Orphan path!")
            .file_name()
            .expect("Dir with no name!")
            .to_str()
            .expect("Invalid Unicode");

        let index = path
            .file_stem()
            .expect("Not file stem")
            .to_str()
            .expect("Invalid Unicode")
            .parse::<usize>()
            .expect("Not a number");

        let buf_i = index - self.node_mint_count;

        if let Some(node) = self.video_nodes.get_mut(buf_i) {
            node.qualities
                .insert(quality.to_owned(), IPLDLink { link: cid });

            node.setup = self.setup_link;

            if buf_i == 0 {
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

        self.mint_video_node().await;
    }

    async fn mint_video_node(&mut self) {
        let node = match self.video_nodes.front() {
            Some(node) => node,
            None => return,
        };

        if node.setup.is_none() {
            return;
        }

        if node.qualities.len() != self.setup_count {
            return;
        }

        if node.previous.is_none() && self.node_mint_count > 0 {
            return;
        }

        let cid = match ipfs_dag_put_node_async(&self.ipfs, node).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("IPFS: dag put failed {}", e);
                return;
            }
        };

        self.video_nodes.pop_front();
        self.node_mint_count += 1;
        self.previous = Some(IPLDLink { link: cid });

        let msg = Archive::Video(cid);

        if let Err(error) = self.archive_tx.send(msg).await {
            eprintln!("Archive receiver hung up! Error: {}", error);
        }

        match self.ipfs.pubsub_pub(&self.topic, &cid.to_string()).await {
            Ok(_) => println!("IPFS: GossipSub published => {}", &cid),
            Err(e) => eprintln!("IPFS: pubsub pub failed {}", e),
        }
    }
}
