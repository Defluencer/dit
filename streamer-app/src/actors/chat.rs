use crate::actors::archivist::Archive;
use crate::dag_nodes::IPLDLink;

use std::collections::HashSet;
use std::str;

use tokio::stream::StreamExt;
use tokio::sync::mpsc::Sender;

//use hyper::body::Bytes;

//use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use serde::{Deserialize, Serialize};

//use cid::Cid;
use multibase::Base;

//TODO check if BrightID can be integrated

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct ChatIdentity {
    #[serde(rename = "key")]
    pub public_key: String,
}

/// Chat message optionaly signed with some form of private key
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub identity: ChatIdentity,

    pub signature: String,

    pub data: ChatContent,
}

/// User name, message and a link to VideoNode as timestamp
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatContent {
    pub name: String,

    pub message: String,

    pub timestamp: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Blacklist {
    pub blacklist: HashSet<ChatIdentity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Whitelist {
    pub whitelist: HashSet<ChatIdentity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Moderators {
    pub mods: HashSet<ChatIdentity>,
}

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
        let encoded = response.data.as_ref()?;

        let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Decoding message failed");

        let msg_str = match str::from_utf8(&decoded) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("Chat message invalid UTF-8");
                return None;
            }
        };

        let chat_message = match serde_json::from_str(msg_str) {
            Ok(data) => data,
            Err(_) => {
                eprintln!("Chat message deserialization failed");
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
