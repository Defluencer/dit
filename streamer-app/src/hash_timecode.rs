use crate::config::Config;
use crate::dag_nodes::{HoursNode, IPLDLink, MinutesNode, SecondsNode, StreamNode};

use std::convert::TryFrom;
use std::io::Cursor;

use tokio::sync::mpsc::Receiver;

use ipfs_api::IpfsClient;

use cid::Cid;

pub enum Timecode {
    Add(Cid),
    Finalize,
}

pub struct HashTimecode {
    ipfs: IpfsClient,

    timecode_rx: Receiver<Timecode>,

    config: Config,

    seconds_node: SecondsNode,
    minutes_node: MinutesNode,
    hours_node: HoursNode,
}

impl HashTimecode {
    pub fn new(ipfs: IpfsClient, timecode_rx: Receiver<Timecode>, config: Config) -> Self {
        Self {
            ipfs,

            timecode_rx,

            config,

            seconds_node: SecondsNode {
                links_to_video: Vec::with_capacity(60),
            },
            minutes_node: MinutesNode {
                links_to_seconds: Vec::with_capacity(60),
            },
            hours_node: HoursNode {
                links_to_minutes: Vec::with_capacity(24),
            },
        }
    }

    pub async fn collect(&mut self) {
        while let Some(event) = self.timecode_rx.recv().await {
            match event {
                Timecode::Add(cid) => self.add_segment(cid).await,
                Timecode::Finalize => self.finalize().await,
            }
        }
    }

    async fn add_segment(&mut self, cid: Cid) {
        let link = IPLDLink { link: cid };

        for _ in 0..self.config.video_segment_duration {
            self.seconds_node.links_to_video.push(link.clone());
        }

        if self.seconds_node.links_to_video.len() < 60 {
            return;
        }

        self.collect_seconds().await;

        if self.minutes_node.links_to_seconds.len() < 60 {
            return;
        }

        self.collect_minutes().await;
    }

    async fn collect_seconds(&mut self) {
        #[cfg(debug_assertions)]
        println!(
            "{}",
            serde_json::to_string_pretty(&self.seconds_node).unwrap()
        );

        let json_string =
            serde_json::to_string(&self.seconds_node).expect("Can't serialize seconds node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => {
                Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed")
            }
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        self.seconds_node.links_to_video.clear();

        let link = IPLDLink { link: cid };

        self.minutes_node.links_to_seconds.push(link);
    }

    async fn collect_minutes(&mut self) {
        #[cfg(debug_assertions)]
        println!(
            "{}",
            serde_json::to_string_pretty(&self.minutes_node).unwrap()
        );

        let json_string =
            serde_json::to_string(&self.minutes_node).expect("Can't serialize minutes node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => {
                Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed")
            }
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        self.minutes_node.links_to_seconds.clear();

        let link = IPLDLink { link: cid };

        self.hours_node.links_to_minutes.push(link);
    }

    async fn finalize(&mut self) {
        println!("Finalizing Stream...");

        if !self.seconds_node.links_to_video.is_empty() {
            self.collect_seconds().await;
        }

        if !self.minutes_node.links_to_seconds.is_empty() {
            self.collect_minutes().await;
        }

        #[cfg(debug_assertions)]
        println!(
            "{}",
            serde_json::to_string_pretty(&self.hours_node).unwrap()
        );

        let json_string =
            serde_json::to_string(&self.hours_node).expect("Can't serialize hours node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => {
                Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed")
            }
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        let stream = StreamNode {
            timecode: IPLDLink { link: cid },
        };

        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(&stream).unwrap());

        let json_string = serde_json::to_string(&stream).expect("Can't serialize stream node");

        let stream_cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        if self.config.pin_stream {
            match self.ipfs.pin_add(&stream_cid, true).await {
                Ok(_) => println!("Stream CID => {}", &stream_cid),
                Err(e) => eprintln!("IPFS pin add failed {}", e),
            }
        }
    }
}
