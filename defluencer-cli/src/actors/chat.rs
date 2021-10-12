use crate::actors::archivist::Archive;
use crate::cli::moderation::{BANS_KEY, MODS_KEY};
use crate::utils::config::ChatConfig;
use crate::utils::dag_nodes::{
    get_from_ipns, ipfs_dag_get_node_async, ipfs_dag_put_node_async, update_ipns,
};

use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

use ipfs_api::response::{Error, PubsubSubResponse};
use ipfs_api::IpfsClient;

use linked_data::chat::{ChatId, Message, MessageType};
use linked_data::moderation::{Ban, Bans, ChatModerationCache, Moderators};
use linked_data::signature::SignedMessage;
use linked_data::PeerId;

pub struct ChatAggregator {
    ipfs: IpfsClient,

    archive_tx: UnboundedSender<Archive>,

    mod_db: ChatModerationCache,

    topic: String,

    bans: Bans,

    new_ban_count: usize,

    mods: Moderators,
}

impl ChatAggregator {
    pub async fn new(
        ipfs: IpfsClient,
        archive_tx: UnboundedSender<Archive>,
        config: ChatConfig,
    ) -> Result<Self, Error> {
        let ChatConfig { topic } = config;

        let ((_, mods), (_, bans)) = match tokio::try_join!(
            get_from_ipns(&ipfs, MODS_KEY),
            get_from_ipns(&ipfs, BANS_KEY)
        ) {
            Ok(res) => res,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(Self {
            ipfs,

            archive_tx,

            mod_db: ChatModerationCache::new(100, 0),

            topic,

            bans,

            new_ban_count: 0,

            mods,
        })
    }

    pub async fn start(&mut self) {
        let mut stream = self.ipfs.pubsub_sub(&self.topic, true);

        println!("✅ Chat System Online");

        while let Some(result) = stream.next().await {
            if self.archive_tx.is_closed() {
                //Hacky way to shutdown
                break;
            }

            match result {
                Ok(response) => self.on_pubsub_message(response).await,
                Err(error) => {
                    eprintln!("{}", error);
                    continue;
                }
            }
        }

        if self.new_ban_count > 0 {
            println!(
                "Updating Banned List with {} New Users 👍",
                self.new_ban_count
            );

            if let Err(e) = update_ipns(&self.ipfs, BANS_KEY, &self.bans).await {
                eprintln!("❗ IPNS Update Failed. {}", e);
            }
        }

        println!("❌ Chat System Offline");
    }

    async fn on_pubsub_message(&mut self, msg: PubsubSubResponse) {
        let peer = match msg.from {
            Some(from) => from,
            None => return,
        };

        if self.mod_db.is_banned(&peer) {
            return;
        }

        let data = match msg.data {
            Some(data) => data,
            None => return,
        };

        let msg: Message = match serde_json::from_slice(&data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❗ PubSub Message Deserialization Failed. {}", e);
                return;
            }
        };

        if !self.mod_db.is_verified(&peer, &msg.sig.link) {
            return self.get_origin(peer, msg).await;
        }

        self.process_msg(&peer, msg).await
    }

    async fn get_origin(&mut self, peer: PeerId, msg: Message) {
        let sign_msg: SignedMessage<ChatId> =
            match ipfs_dag_get_node_async(&self.ipfs, &msg.sig.link.to_string()).await {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("❗ IPFS: dag get failed {}", e);
                    return;
                }
            };

        self.mod_db
            .add_peer(&peer, msg.sig.link, sign_msg.address, None);

        if peer != sign_msg.data.peer_id {
            self.mod_db.ban_peer(&peer);
            return;
        }

        if !sign_msg.verify() {
            self.mod_db.ban_peer(&peer);
            return;
        }

        if self.bans.banned.contains(&sign_msg.address) {
            self.mod_db.ban_peer(&peer);
            return;
        }

        self.process_msg(&peer, msg).await
    }

    async fn process_msg(&mut self, peer: &str, msg: Message) {
        match msg.msg {
            MessageType::Chat(unmsg) => self.mint_and_archive(&unmsg).await,
            MessageType::Ban(ban) => self.update_bans(peer, ban),
            MessageType::Mod(_) => {}
        }
    }

    async fn mint_and_archive(&mut self, msg: &str) {
        let cid = match ipfs_dag_put_node_async(&self.ipfs, &msg).await {
            Ok(cid) => cid,
            Err(e) => {
                eprintln!("❗ IPFS: dag put failed {}", e);
                return;
            }
        };

        let msg = Archive::Chat(cid);

        if let Err(error) = self.archive_tx.send(msg) {
            eprintln!("❗ Archive receiver hung up. {}", error);
        }
    }

    fn update_bans(&mut self, peer: &str, ban: Ban) {
        let address = self.mod_db.get_address(peer).unwrap();

        if !self.mods.mods.contains(address) {
            return;
        }

        self.mod_db.ban_peer(&ban.peer_id);
        self.bans.banned.insert(ban.address);

        self.new_ban_count += 1;
    }
}
