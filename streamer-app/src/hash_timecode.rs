use crate::config::Config;

use std::io::Cursor;

use tokio::sync::mpsc::Receiver;

use serde::Serialize;

use ipfs_api::IpfsClient;

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

    async fn add_segment(&mut self, cid: String) {
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
        let json_string =
            serde_json::to_string(&self.seconds_node).expect("Can't serialize seconds node");

        #[cfg(debug_assertions)]
        println!("{:#}", &json_string);

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        //let cid = Cid::from_str(&cid).expect("CID from dag put response failed");

        self.seconds_node.links_to_video.clear();

        let link = IPLDLink { link: cid };

        self.minutes_node.links_to_seconds.push(link);
    }

    async fn collect_minutes(&mut self) {
        let json_string =
            serde_json::to_string(&self.minutes_node).expect("Can't serialize minutes node");

        #[cfg(debug_assertions)]
        println!("{:#}", &json_string);

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        //let cid = Cid::from_str(&cid).expect("CID from dag put response failed");

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

        let json_string =
            serde_json::to_string(&self.hours_node).expect("Can't serialize minutes node");

        #[cfg(debug_assertions)]
        println!("{:#}", &json_string);

        let hours_node_cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => response.cid.cid_string,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        let stream = Stream {
            timecode: IPLDLink {
                link: hours_node_cid,
            },
        };

        let json_string = serde_json::to_string(&stream).expect("Can't serialize minutes node");

        #[cfg(debug_assertions)]
        println!("{:#}", &json_string);

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

pub enum Timecode {
    Add(String),
    Finalize,
}

#[derive(Serialize, Debug, Clone)]
pub struct IPLDLink {
    #[serde(rename = "/")]
    pub link: String, //TODO find a way to serialize Cid instead of String
}

/// Root CID.
#[derive(Serialize, Debug)]
struct Stream {
    #[serde(rename = "time")]
    timecode: IPLDLink, // ../<StreamHash>/time/..
}

/// Links all hour nodes for multiple hours of video.
#[derive(Serialize, Debug)]
struct HoursNode {
    #[serde(rename = "hour")]
    links_to_minutes: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/..
}

/// Links all minute nodes for 1 hour of video.
#[derive(Serialize, Debug)]
struct MinutesNode {
    #[serde(rename = "minute")]
    links_to_seconds: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/..
}

/// Links all variants nodes for 1 minute of video.
#[derive(Serialize, Debug)]
struct SecondsNode {
    #[serde(rename = "second")]
    links_to_video: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/..
}
