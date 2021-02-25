use futures_util::StreamExt;

use yew::services::ConsoleService;
use yew::Callback;

use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use linked_data::chat::{ChatContent, ChatIdentity, ChatMessage};

pub struct LiveChatManager {
    topic: String,

    ipfs: IpfsClient,

    cb: Callback<ChatContent>,
}

impl LiveChatManager {
    pub fn new(topic: String, cb: Callback<ChatContent>) -> Self {
        let ipfs = IpfsClient::default();

        Self { topic, ipfs, cb }
    }

    async fn load_live_chat(&mut self) {
        let mut stream = self.ipfs.pubsub_sub(&self.topic, true);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => self.process_msg(&response),
                Err(error) => {
                    eprintln!("{}", error);
                    continue;
                }
            }
        }
    }

    fn process_msg(&mut self, msg: &PubsubSubResponse) {
        let decoded = match msg.data.as_ref() {
            Some(data) => data,
            None => return,
        };

        let chat_message: ChatMessage = match serde_json::from_slice(decoded) {
            Ok(msg) => msg,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let content = chat_message.data;

        self.cb.emit(content);
    }

    pub async fn send_chat(&self, msg: &str) {
        //ipfs_publish(&self.topic, msg);

        match self.ipfs.pubsub_pub(&self.topic, msg).await {
            Ok(_) => {}
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        }
    }
}
/*
impl Drop for LiveChatManager {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping LiveChatManager");

        ipfs_unsubscribe(&self.topic);
    }
} */
