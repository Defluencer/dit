//use crate::chat::ChatMessage;
//use crate::config::get_config;
use crate::dag_nodes::IPLDLink;

use std::collections::VecDeque;
use std::convert::TryFrom;
use std::io::Cursor;

use tokio::sync::mpsc::Receiver;

use ipfs_api::IpfsClient;

use serde::Serialize;

use cid::Cid;

/// Stream Root CID.
#[derive(Serialize, Debug)]
pub struct StreamNode {
    #[serde(rename = "time")]
    pub timecode: IPLDLink, // ../<StreamHash>/time/..
}

/// Links all hour nodes for multiple hours of video.
#[derive(Serialize, Debug)]
pub struct DayNode {
    #[serde(rename = "hour")]
    pub links_to_hours: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/..
}

/// Links all minute nodes for 1 hour of video.
#[derive(Serialize, Debug)]
pub struct HourNode {
    #[serde(rename = "minute")]
    pub links_to_minutes: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/..
}

/// Links all variants nodes for 1 minute of video.
#[derive(Serialize, Debug)]
pub struct MinuteNode {
    #[serde(rename = "second")]
    pub links_to_seconds: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/..
}

/// Links video and chat nodes.
#[derive(Serialize, Debug)]
pub struct SecondNode {
    #[serde(rename = "video")]
    pub link_to_video: IPLDLink, // ../<StreamHash>/time/hour/1/minute/15/second/30/video/..

    #[serde(rename = "chat")]
    pub links_to_chat: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/chat/0/..
}

pub enum Archive {
    //Chat(ChatMessage),
    Video(Cid),
    Finalize,
}

pub struct Chronicler {
    ipfs: IpfsClient,

    archive_rx: Receiver<Archive>,

    video_chat_buffer: VecDeque<SecondNode>,

    minute_node: MinuteNode,
    hour_node: HourNode,
    day_node: DayNode,

    video_segment_duration: usize,
}

impl Chronicler {
    pub async fn new(ipfs: IpfsClient, archive_rx: Receiver<Archive>) -> Self {
        //let config = get_config(&ipfs).await;

        Self {
            ipfs,

            archive_rx,

            video_chat_buffer: VecDeque::with_capacity(
                120 / 4, /* config.video_segment_duration */
            ), //120 == 2 minutes

            minute_node: MinuteNode {
                links_to_seconds: Vec::with_capacity(60),
            },

            hour_node: HourNode {
                links_to_minutes: Vec::with_capacity(60),
            },

            day_node: DayNode {
                links_to_hours: Vec::with_capacity(24),
            },

            video_segment_duration: 4, /* config.video_segment_duration */
        }
    }

    pub async fn collect(&mut self) {
        while let Some(event) = self.archive_rx.recv().await {
            match event {
                //Archive::Chat(msg) => self.archive_chat_message(msg).await,
                Archive::Video(cid) => self.archive_video_segment(cid).await,
                Archive::Finalize => self.finalize().await,
            }
        }
    }

    /// Link chat message to SecondNodes.
    /* async fn archive_chat_message(&mut self, msg: ChatMessage) {
        for node in self.video_chat_buffer.iter_mut() {
            if node.link_to_video != msg.data.timestamp {
                continue;
            }

            let json_string = serde_json::to_string(&msg).expect("Can't serialize chat msg");

            let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
                Ok(response) => Cid::try_from(response.cid.cid_string)
                    .expect("CID from dag put response failed"),
                Err(e) => {
                    eprintln!("IPFS dag put failed {}", e);
                    return;
                }
            };

            let link = IPLDLink { link: cid };

            node.links_to_chat.push(link);

            break;
        }
    } */

    /// Buffers SecondNodes, waiting for chat messages to be linked.
    async fn archive_video_segment(&mut self, cid: Cid) {
        let link_variants = IPLDLink { link: cid };

        let second_node = SecondNode {
            link_to_video: link_variants,
            links_to_chat: Vec::with_capacity(5),
        };

        self.video_chat_buffer.push_back(second_node);

        if self.video_chat_buffer.len() < self.video_chat_buffer.capacity() {
            return;
        }

        self.collect_second().await;

        if self.minute_node.links_to_seconds.len() < self.minute_node.links_to_seconds.capacity() {
            return;
        }

        self.collect_minute().await;

        if self.hour_node.links_to_minutes.len() < self.hour_node.links_to_minutes.capacity() {
            return;
        }

        self.collect_hour().await;
    }

    /// Create DAG node containing a link to video segment and all chat messages.
    /// MinuteNode is then appended with the CID.
    async fn collect_second(&mut self) {
        let second_node = self.video_chat_buffer.pop_front().unwrap();

        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(&second_node).unwrap());

        let json_string = serde_json::to_string(&second_node).expect("Can't serialize second node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => {
                Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed")
            }
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        let link = IPLDLink { link: cid };

        for _ in 0..self.video_segment_duration {
            self.minute_node.links_to_seconds.push(link.clone());
        }
    }

    /// Create DAG node containing 60 SecondNode links. HourNode is then appended with the CID.
    async fn collect_minute(&mut self) {
        let node = &self.minute_node;

        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(node).unwrap());

        let json_string = serde_json::to_string(node).expect("Can't serialize seconds node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => {
                Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed")
            }
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        self.minute_node.links_to_seconds.clear();

        let link = IPLDLink { link: cid };

        self.hour_node.links_to_minutes.push(link);
    }

    /// Create DAG node containing 60 MinuteNode links. DayNode is then appended with the CID.
    async fn collect_hour(&mut self) {
        let node = &self.hour_node;

        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(node).unwrap());

        let json_string = serde_json::to_string(node).expect("Can't serialize minutes node");

        let cid = match self.ipfs.dag_put(Cursor::new(json_string)).await {
            Ok(response) => {
                Cid::try_from(response.cid.cid_string).expect("CID from dag put response failed")
            }
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        self.hour_node.links_to_minutes.clear();

        let link = IPLDLink { link: cid };

        self.day_node.links_to_hours.push(link);
    }

    /// Create all remaining DAG nodes then pin and print the final stream CID.
    async fn finalize(&mut self) {
        println!("Finalizing Stream...");

        while !self.video_chat_buffer.is_empty() {
            self.collect_second().await;
        }

        if !self.minute_node.links_to_seconds.is_empty() {
            self.collect_minute().await;
        }

        if !self.hour_node.links_to_minutes.is_empty() {
            self.collect_hour().await;
        }

        let node = &self.day_node;

        #[cfg(debug_assertions)]
        println!("{}", serde_json::to_string_pretty(node).unwrap());

        let json_string = serde_json::to_string(node).expect("Can't serialize hours node");

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

        match self.ipfs.pin_add(&stream_cid, true).await {
            Ok(_) => println!("Stream CID => {}", &stream_cid),
            Err(e) => eprintln!("IPFS pin add failed {}", e),
        }
    }
}
