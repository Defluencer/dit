use crate::actors::archivist::Archive;

use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

//use hyper::body::Bytes;

//use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use linked_data::chat::{ChatIdentity, ChatMessage};
use linked_data::config::ChatConfig;

//use cid::Cid;

//TODO check if BrightID can be integrated

pub struct ChatAggregator {
    ipfs: IpfsClient,

    archive_tx: UnboundedSender<Archive>,

    config: ChatConfig,
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
            //blacklist,

            //whitelist,

            //mods: mods,
        }
    }

    pub async fn start_receiving(&mut self) {
        let mut stream = self.ipfs.pubsub_sub(&self.config.pubsub_topic, true);

        println!("Chat System Online");

        while let Some(result) = stream.next().await {
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
        let data = match msg.data.as_ref() {
            Some(data) => data,
            None => return,
        };

        let chat_message = match serde_json::from_slice(data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Deserialization failed. {}", e);
                return;
            }
        };

        if !self.is_auth_signature(&chat_message) {
            return;
        }

        if !self.is_allowed(&chat_message.identity) {
            return;
        }

        let msg = Archive::Chat(chat_message);

        if let Err(error) = self.archive_tx.send(msg) {
            eprintln!("Archive receiver hung up. {}", error);
        }
    }

    /// Verify signature authenticity
    fn is_auth_signature(&self, _msg: &ChatMessage) -> bool {
        //TODO verify signature
        true
    }

    /// Verify identity against white & black lists
    fn is_allowed(&self, _identity: &ChatIdentity) -> bool {
        //TODO verify white & black list
        true
        //self.whitelist.whitelist.contains(identity) || !self.blacklist.blacklist.contains(identity)
    }
}
