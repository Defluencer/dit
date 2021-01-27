use crate::utils::ipfs_dag_put_node_async;

use std::collections::VecDeque;
use std::convert::TryFrom;
use std::io::Cursor;

use tokio::sync::mpsc::Receiver;

use ipfs_api::IpfsClient;

use linked_data::chat::ChatMessage;
use linked_data::stream::{DayNode, HourNode, MinuteNode, SecondNode, StreamNode};
use linked_data::IPLDLink;

use cid::Cid;

pub enum Archive {
    Chat(ChatMessage),
    Video(Cid),
    Finalize,
}

pub struct Archivist {
    ipfs: IpfsClient,

    archive_rx: Receiver<Archive>,

    buffer_cap: usize,
    video_chat_buffer: VecDeque<SecondNode>,

    minute_node: MinuteNode,
    hour_node: HourNode,
    day_node: DayNode,

    segment_duration: usize,
}

impl Archivist {
    pub fn new(ipfs: IpfsClient, archive_rx: Receiver<Archive>, segment_duration: usize) -> Self {
        let buffer_cap = 60 / segment_duration; // 1 minutes

        Self {
            ipfs,

            archive_rx,

            buffer_cap,
            video_chat_buffer: VecDeque::with_capacity(buffer_cap),

            minute_node: MinuteNode {
                links_to_seconds: Vec::with_capacity(60),
            },

            hour_node: HourNode {
                links_to_minutes: Vec::with_capacity(60),
            },

            day_node: DayNode {
                links_to_hours: Vec::with_capacity(24),
            },

            segment_duration,
        }
    }

    pub async fn collect(&mut self) {
        println!("Archive System Online");

        while let Some(event) = self.archive_rx.recv().await {
            match event {
                Archive::Chat(msg) => self.archive_chat_message(msg).await,
                Archive::Video(cid) => self.archive_video_segment(cid).await,
                Archive::Finalize => self.finalize().await,
            }
        }
    }

    /// Link chat message to SecondNodes.
    async fn archive_chat_message(&mut self, msg: ChatMessage) {
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
    }

    /// Buffers SecondNodes, waiting for chat messages to be linked.
    async fn archive_video_segment(&mut self, cid: Cid) {
        let link_variants = IPLDLink { link: cid };

        let second_node = SecondNode {
            link_to_video: link_variants,
            links_to_chat: Vec::with_capacity(5),
        };

        self.video_chat_buffer.push_back(second_node);

        if self.video_chat_buffer.len() < self.buffer_cap {
            return;
        }

        self.collect_second().await;

        if self.minute_node.links_to_seconds.len() < 60 {
            return;
        }

        self.collect_minute().await;

        if self.hour_node.links_to_minutes.len() < 60 {
            return;
        }

        self.collect_hour().await;
    }

    /// Create DAG node containing a link to video segment and all chat messages.
    /// MinuteNode is then appended with the CID.
    async fn collect_second(&mut self) {
        let node = self.video_chat_buffer.pop_front().unwrap();

        let cid = match ipfs_dag_put_node_async(&self.ipfs, &node).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        let link = IPLDLink { link: cid };

        //since duration > 1 sec
        for _ in 0..self.segment_duration {
            self.minute_node.links_to_seconds.push(link.clone());
        }

        //self.minute_node.links_to_seconds.push(link);
    }

    /// Create DAG node containing 60 SecondNode links. HourNode is then appended with the CID.
    async fn collect_minute(&mut self) {
        let cid = match ipfs_dag_put_node_async(&self.ipfs, &self.minute_node).await {
            Ok(cid) => cid,
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
        let cid = match ipfs_dag_put_node_async(&self.ipfs, &self.hour_node).await {
            Ok(cid) => cid,
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

            if self.minute_node.links_to_seconds.len() >= 60 {
                self.collect_minute().await;
            }

            if self.hour_node.links_to_minutes.len() >= 60 {
                self.collect_hour().await;
            }
        }

        if !self.minute_node.links_to_seconds.is_empty() {
            self.collect_minute().await;
        }

        if !self.hour_node.links_to_minutes.is_empty() {
            self.collect_hour().await;
        }

        let cid = match ipfs_dag_put_node_async(&self.ipfs, &self.day_node).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        let stream = StreamNode {
            timecode: IPLDLink { link: cid },
        };

        let cid = match ipfs_dag_put_node_async(&self.ipfs, &stream).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("IPFS dag put failed {}", e);
                return;
            }
        };

        match self.ipfs.pin_add(&cid.to_string(), true).await {
            Ok(_) => println!("Stream CID => {}", &cid.to_string()),
            Err(e) => eprintln!("IPFS pin add failed {}", e),
        }
    }
}
