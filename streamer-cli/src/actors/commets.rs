use crate::actors::VideoData;
use crate::utils::dag_nodes::ipfs_dag_put_node_async;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use ipfs_api::IpfsClient;

use linked_data::IPLDLink;

use cid::Cid;

pub struct CommentsAggregator {
    ipfs: IpfsClient,
}

impl CommentsAggregator {
    pub fn new(ipfs: IpfsClient) -> Self {
        Self { ipfs }
    }

    pub async fn start(&mut self) {
        println!("✅ Comments System Online");

        /* while let Some(msg) = self.service_rx.recv().await {
        } */

        println!("❌ Comments System Offline");
    }
}

// Listen for message on topic
// Verify comment signature
// Get index of comment origin from content feed
// Get comments at index and add new comment
// Dag Put & IPNS update

// To display comments iterate in reverse but skip replies to other comments, save for next step
// Display replies and repeat until no more comments.
