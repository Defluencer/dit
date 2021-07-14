use crate::actors::VideoData;
use crate::utils::dag_nodes::ipfs_dag_put_node_async;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use ipfs_api::IpfsClient;

use linked_data::IPLDLink;

use cid::Cid;

pub struct BlogAggregator {
    ipfs: IpfsClient,
}

impl BlogAggregator {
    pub fn new(ipfs: IpfsClient) -> Self {
        Self { ipfs }
    }

    pub async fn start(&mut self) {
        println!("✅ Blog System Online");

        /* while let Some(msg) = self.service_rx.recv().await {
        } */

        println!("❌ Blog System Offline");
    }
}
