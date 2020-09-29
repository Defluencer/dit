use crate::chronicler::Archive;
use crate::dag_nodes::IPLDLink;

use std::collections::HashSet;
use std::str;

use tokio::stream::StreamExt;
use tokio::sync::mpsc::Sender;

use hyper::body::Bytes;

use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use serde::{Deserialize, Serialize};

use cid::Cid;
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

    blacklist: Blacklist,

    whitelist: Whitelist,

    _mods: Moderators,
}

impl ChatAggregator {
    pub async fn new(ipfs: IpfsClient, archive_tx: Sender<Archive>) -> Self {
        let config = crate::config::get_config(&ipfs).await;

        let blacklist = get_blacklist(&ipfs, config.blacklist.link).await;

        let whitelist = get_whitelist(&ipfs, config.whitelist.link).await;

        let mods = get_mods(&ipfs, config.mods.link).await;

        Self {
            ipfs,

            archive_tx,

            gossipsub_topic: config.gossipsub_topics.chat,

            blacklist,

            whitelist,

            _mods: mods,
        }
    }

    pub async fn aggregate(&mut self) {
        let topic = &self.gossipsub_topic;

        let mut stream = self.ipfs.pubsub_sub(topic, true);

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

    fn is_auth_signature(&self, _msg: &ChatMessage) -> bool {
        //TODO
        true
    }

    fn is_allowed(&self, identity: &ChatIdentity) -> bool {
        self.whitelist.whitelist.contains(identity) || !self.blacklist.blacklist.contains(identity)
    }
}

async fn get_whitelist(ipfs: &IpfsClient, cid: Cid) -> Whitelist {
    let buffer: Result<Bytes, Error> = ipfs.dag_get(&cid.to_string()).collect().await;

    let buffer = buffer.expect("IPFS DAG get failed");

    serde_json::from_slice(&buffer).expect("Deserializing config failed")
}

async fn get_blacklist(ipfs: &IpfsClient, cid: Cid) -> Blacklist {
    let buffer: Result<Bytes, Error> = ipfs.dag_get(&cid.to_string()).collect().await;

    let buffer = buffer.expect("IPFS DAG get failed");

    serde_json::from_slice(&buffer).expect("Deserializing config failed")
}

async fn get_mods(ipfs: &IpfsClient, cid: Cid) -> Moderators {
    let buffer: Result<Bytes, Error> = ipfs.dag_get(&cid.to_string()).collect().await;

    let buffer = buffer.expect("IPFS DAG get failed");

    serde_json::from_slice(&buffer).expect("Deserializing config failed")
}
