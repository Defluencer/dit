use crate::chronicler::Archive;
use crate::dag_nodes::{Blacklist, ChatMessage, Moderators, Whitelist};

use std::convert::TryFrom;
use std::str;

use tokio::stream::StreamExt;
use tokio::sync::mpsc::Sender;

use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use cid::Cid;
use multibase::Base;

pub struct ChatAggregator {
    ipfs: IpfsClient,

    archive_tx: Sender<Archive>,

    gossipsub_topic: String,
}

impl ChatAggregator {
    pub async fn new(ipfs: IpfsClient, archive_tx: Sender<Archive>) -> Self {
        let config = crate::config::get_config(&ipfs).await;

        Self {
            ipfs,

            archive_tx,

            gossipsub_topic: config.gossipsub_topics.chat,
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
        if !self.is_verified_sender(msg) {
            return;
        }

        let chat_message = match decode_message(msg) {
            Some(data) => data,
            None => return,
        };

        if !is_auth_signature(&chat_message) {
            return;
        }

        let msg = Archive::Chat(chat_message);

        if let Err(error) = self.archive_tx.send(msg).await {
            eprintln!("Archive receiver hung up {}", error);
        }
    }

    fn is_verified_sender(&self, response: &PubsubSubResponse) -> bool {
        let encoded = match response.from.as_ref() {
            Some(sender) => sender,
            None => return false,
        };

        let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Decoding sender failed");

        let cid = Cid::try_from(decoded).expect("CID from decoded sender failed");

        #[cfg(debug_assertions)]
        println!("Sender => {}", cid);

        //TODO check peer id whitelist then blacklist

        true
    }
}

fn decode_message(response: &PubsubSubResponse) -> Option<ChatMessage> {
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

fn is_auth_signature(_msg: &ChatMessage) -> bool {
    //TODO
    true
}
