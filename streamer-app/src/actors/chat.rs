use crate::actors::archivist::Archive;
use crate::utils::dag_nodes::{ipfs_dag_get_node_async, ipfs_dag_put_node_async};

use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

//use hyper::body::Bytes;

//use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use linked_data::chat::{SignedMessage, UnsignedMessage};
use linked_data::config::ChatConfig;
use linked_data::moderation::{Blacklist, Moderators, Whitelist};

use cid::Cid;

pub struct ChatAggregator {
    ipfs: IpfsClient,

    archive_tx: UnboundedSender<Archive>,

    config: ChatConfig,

    trusted: HashMap<String, Cid>,
    //blacklist: Blacklist,

    //whitelist: Whitelist,

    //mods: Moderators,
}

impl ChatAggregator {
    pub fn new(ipfs: IpfsClient, archive_tx: UnboundedSender<Archive>, config: ChatConfig) -> Self {
        Self {
            ipfs,

            archive_tx,

            config,

            trusted: HashMap::with_capacity(100),
            //blacklist,

            //whitelist,

            //mods: mods,
        }
    }

    pub async fn start(&mut self) {
        let mut stream = self.ipfs.pubsub_sub(&self.config.pubsub_topic, true);

        println!("Chat System Online");

        while let Some(result) = stream.next().await {
            if self.archive_tx.is_closed() {
                //Hacky way to shutdown
                break;
            }

            match result {
                Ok(response) => self.process_msg(&response).await,
                Err(error) => {
                    eprintln!("{}", error);
                    continue;
                }
            }
        }

        println!("Chat System Offline");
    }

    async fn process_msg(&mut self, msg: &PubsubSubResponse) {
        let from = match msg.from.as_ref() {
            Some(from) => from,
            None => return,
        };

        let data = match msg.data.as_ref() {
            Some(data) => data,
            None => return,
        };

        let msg: UnsignedMessage = match serde_json::from_slice(data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Deserialization failed. {}", e);
                return;
            }
        };

        //TODO filter with lists

        if Some(&msg.origin.link) == self.trusted.get(from) {
            return self.mint_and_archive(msg).await;
        }

        let sign_msg: SignedMessage =
            match ipfs_dag_get_node_async(&self.ipfs, &msg.origin.link.to_string()).await {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("IPFS: dag get failed {}", e);
                    return;
                }
            };

        if from != &sign_msg.data.peer_id {
            return;
        }

        if !sign_msg.verify() {
            return;
        }

        self.trusted.insert(from.to_owned(), msg.origin.link);

        self.mint_and_archive(msg).await;
    }

    async fn mint_and_archive(&mut self, msg: UnsignedMessage) {
        let cid = match ipfs_dag_put_node_async(&self.ipfs, &msg).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("IPFS: dag put failed {}", e);
                return;
            }
        };

        let msg = Archive::Chat(cid);

        if let Err(error) = self.archive_tx.send(msg) {
            eprintln!("Archive receiver hung up. {}", error);
        }
    }

    /// Verify identity against white & black lists
    fn is_allowed(&self, _from: &str, _identity: &UnsignedMessage) -> bool {
        //TODO verify white & black list
        true
        //self.whitelist.whitelist.contains(identity) || !self.blacklist.blacklist.contains(identity)
    }
}
