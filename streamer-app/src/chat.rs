use std::convert::TryFrom;
use std::str;

use tokio::stream::StreamExt;

use ipfs_api::response::{Error, PubsubSubResponse};
use ipfs_api::IpfsClient;

use cid::Cid;
use multibase::Base;

use crate::config::Config;
use crate::dag_nodes::{ChatContent, ChatMessage};

pub struct ChatAggregator {
    ipfs: IpfsClient,

    config: Config,
}

impl ChatAggregator {
    pub fn new(ipfs: IpfsClient, config: Config) -> Self {
        Self { ipfs, config }
    }

    pub async fn aggregate(&mut self) {
        let topic = &self.config.gossipsub_topics.chat;

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
        #[cfg(debug_assertions)]
        println!("{:#?}", msg);

        if !is_verified_sender(msg) {
            return;
        }

        let chat_message = match decode_message(msg) {
            Some(data) => data,
            None => return,
        };

        //TODO verify msg signature

        let content = chat_message.data;
    }
}

fn is_verified_sender(response: &PubsubSubResponse) -> bool {
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
