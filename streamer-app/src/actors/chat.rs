use crate::actors::archivist::Archive;
use crate::utils::dag_nodes::{ipfs_dag_get_node_async, ipfs_dag_put_node_async};

use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

//use hyper::body::Bytes;

//use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use linked_data::chat::{ChatMessage, SignedMessage, UnsignedMessage};
use linked_data::config::ChatConfig;

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

        let chat_message: ChatMessage = match serde_json::from_slice(data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Deserialization failed. {}", e);
                return;
            }
        };

        if !self.is_allowed(from, &chat_message) {
            return;
        }

        match chat_message {
            ChatMessage::Signed(signed) => self.process_signed_msg(from, signed).await,
            ChatMessage::Unsigned(unsigned) => self.process_unsigned_msg(from, unsigned).await,
        }
    }

    async fn process_signed_msg(&mut self, from: &str, msg: SignedMessage) {
        if from != msg.data.peer_id {
            return;
        }

        if !msg.verify() {
            return;
        }

        let cid = match ipfs_dag_put_node_async(&self.ipfs, &msg).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("IPFS: dag put failed {}", e);
                return;
            }
        };

        self.trusted.insert(from.to_owned(), cid);

        let msg = Archive::Chat(cid);

        if let Err(error) = self.archive_tx.send(msg) {
            eprintln!("Archive receiver hung up. {}", error);
        }
    }

    async fn process_unsigned_msg(&mut self, from: &str, msg: UnsignedMessage) {
        if Some(&msg.origin.link) != self.trusted.get(from) {
            let msg: SignedMessage =
                match ipfs_dag_get_node_async(&self.ipfs, &msg.origin.link.to_string()).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("IPFS: dag get failed {}", e);
                        return;
                    }
                };

            return self.process_signed_msg(from, msg).await;
        }

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
    fn is_allowed(&self, _from: &str, _identity: &ChatMessage) -> bool {
        //TODO verify white & black list
        true
        //self.whitelist.whitelist.contains(identity) || !self.blacklist.blacklist.contains(identity)
    }
}
