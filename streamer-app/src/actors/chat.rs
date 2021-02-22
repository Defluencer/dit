use crate::actors::archivist::Archive;

use std::str;

use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;

//use hyper::body::Bytes;

//use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use linked_data::chat::{ChatIdentity, ChatMessage};

//use cid::Cid;

//TODO check if BrightID can be integrated

pub struct ChatAggregator {
    ipfs: IpfsClient,

    archive_tx: Sender<Archive>,

    gossipsub_topic: String,
    //blacklist: Blacklist,

    //whitelist: Whitelist,

    //mods: Moderators,
}

impl ChatAggregator {
    pub fn new(ipfs: IpfsClient, archive_tx: Sender<Archive>, gossipsub_topic: String) -> Self {
        Self {
            ipfs,

            archive_tx,

            gossipsub_topic,
            //blacklist,

            //whitelist,

            //mods: mods,
        }
    }

    pub async fn start_receiving(&mut self) {
        let topic = &self.gossipsub_topic;

        let mut stream = self.ipfs.pubsub_sub(topic, true);

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
    }

    async fn process_msg(&mut self, msg: &PubsubSubResponse) {
        let chat_message = match self.decode_message(msg) {
            Some(data) => data,
            None => return,
        };

        if !self.is_auth_signature(&chat_message) {
            return;
        }

        if !self.is_allowed(&chat_message.identity) {
            return;
        }

        let msg = Archive::Chat(chat_message);

        if let Err(error) = self.archive_tx.send(msg).await {
            eprintln!("Archive receiver hung up {}", error);
        }
    }

    /// Decode chat messages from Base64 then serialize to ChatMessage
    fn decode_message(&self, response: &PubsubSubResponse) -> Option<ChatMessage> {
        let decoded = response.data.as_ref()?;

        //let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Decoding message failed");

        let msg_str = match str::from_utf8(decoded) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("Invalid UTF-8");
                return None;
            }
        };

        let chat_message = match serde_json::from_str(msg_str) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("Deserialization failed");
                return None;
            }
        };

        Some(chat_message)
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
