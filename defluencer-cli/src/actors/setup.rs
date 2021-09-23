use crate::actors::VideoData;
use crate::utils::dag_nodes::ipfs_dag_put_node_async;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use ipfs_api::IpfsClient;

use linked_data::video::{SetupNode, Track};
use linked_data::IPLDLink;

use cid::Cid;

use m3u8_rs::playlist::MasterPlaylist;

type TrackData = (Option<String>, Option<usize>, Option<IPLDLink>);

#[derive(Debug)]
pub enum SetupData {
    Playlist(MasterPlaylist),
    Segment((PathBuf, Cid)),
}

pub struct SetupAggregator {
    ipfs: IpfsClient,

    service_rx: UnboundedReceiver<SetupData>,
    video_tx: UnboundedSender<VideoData>,

    track_len: usize,

    map: HashMap<String, TrackData>,
}

impl SetupAggregator {
    pub fn new(
        ipfs: IpfsClient,
        service_rx: UnboundedReceiver<SetupData>,
        video_tx: UnboundedSender<VideoData>,
    ) -> Self {
        Self {
            ipfs,

            service_rx,
            video_tx,

            track_len: 0,

            map: HashMap::with_capacity(4),
        }
    }

    pub async fn start(&mut self) {
        println!("✅ Setup System Online");

        while let Some(msg) = self.service_rx.recv().await {
            match msg {
                SetupData::Playlist(pl) => self.process_master_playlist(pl).await,
                SetupData::Segment((path, cid)) => self.init_seg(path, cid).await,
            }
        }

        println!("❌ Setup System Offline");
    }

    /// Update track with initialization segments then try to mint node.
    async fn init_seg(&mut self, path: PathBuf, cid: Cid) {
        let name = path
            .parent()
            .expect("Orphan path!")
            .file_name()
            .expect("Dir with no name!")
            .to_str()
            .expect("Invalid Unicode");

        let link = Some(cid.into());

        if let Some((_, _, init_seg)) = self.map.get_mut(name) {
            *init_seg = link;
        } else {
            self.map.insert(name.to_owned(), (None, None, link));
        }

        self.try_mint_setup_node().await;
    }

    /// Create or update tracks based on master playlist then try to mint node.
    async fn process_master_playlist(&mut self, pl: MasterPlaylist) {
        #[cfg(debug_assertions)]
        println!("{:#?}", pl);

        self.track_len = pl.variants.len();

        for variant in pl.variants.into_iter().rev() {
            let path = Path::new(&variant.uri);

            let v_name = path
                .parent()
                .expect("Orphan path!")
                .file_name()
                .expect("Dir with no name!")
                .to_str()
                .expect("Invalid Unicode");

            let v_codec = match variant.codecs {
                Some(codec) => {
                    if v_name == "audio" {
                        Some(format!(r#"audio/mp4; codecs="{}""#, codec))
                    } else {
                        Some(format!(r#"video/mp4; codecs="{}""#, codec))
                    }
                }
                None => None,
            };

            let v_bandwidth = variant.bandwidth.parse::<usize>().ok();

            if let Some((codec, bandwidth, _)) = self.map.get_mut(v_name) {
                *codec = v_codec;
                *bandwidth = v_bandwidth;
            } else {
                self.map
                    .insert(v_name.to_owned(), (v_codec, v_bandwidth, None));
            }
        }

        self.try_mint_setup_node().await;
    }

    /// Mint SetupNode if it meets all requirements.
    async fn try_mint_setup_node(&mut self) {
        if self.map.is_empty() {
            return;
        }

        if self.map.len() != self.track_len {
            return;
        }

        for (codec, bandwidth, init_seg) in self.map.values() {
            if codec.is_none() || bandwidth.is_none() || init_seg.is_none() {
                return;
            }
        }

        let mut tracks = Vec::with_capacity(self.track_len);

        for (name, (codec, bandwidth, init_seg)) in self.map.drain() {
            let codec = codec.unwrap();
            let bandwidth = bandwidth.unwrap();
            let initialization_segment = init_seg.unwrap();

            let track = Track {
                name,
                codec,
                initialization_segment,
                bandwidth,
            };

            tracks.push(track);
        }

        tracks.sort_unstable_by_key(|track| track.bandwidth);

        let setup_node = SetupNode { tracks };

        let cid = ipfs_dag_put_node_async(&self.ipfs, &setup_node)
            .await
            .expect("IPFS: SetupNode dag put failed"); // Panic because can't be recovered from anyway

        println!("Setup Node Minted => {}", &cid.to_string());

        let msg = VideoData::Setup((cid.into(), self.track_len));

        if let Err(error) = self.video_tx.send(msg) {
            eprintln!("❗ Video receiver hung up! Error: {}", error);
        }

        self.service_rx.close();
    }
}
