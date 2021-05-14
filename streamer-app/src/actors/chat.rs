use crate::actors::archivist::Archive;
use crate::cli::moderation::BANS_KEY;
use crate::utils::dag_nodes::{ipfs_dag_get_node_async, ipfs_dag_put_node_async, update_ipns};

use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;

use linked_data::chat::{Address, SignedMessage, UnsignedMessage};
use linked_data::config::ChatConfig;
use linked_data::moderation::{Ban, Bans, Moderators};
use linked_data::{Message, MessageType};

use cid::Cid;

pub struct ChatAggregator {
    ipfs: IpfsClient,

    archive_tx: UnboundedSender<Archive>,

    /// Map of peer IDs to signed message CIDs
    trusted: HashMap<String, (Cid, Address)>,

    /// Set of banned peer IDs.
    ban_cache: HashSet<String>,
    ban_count: usize,

    topic: String,

    bans: Bans,
    mods: Moderators,
}

impl ChatAggregator {
    pub async fn new(
        ipfs: IpfsClient,
        archive_tx: UnboundedSender<Archive>,
        config: ChatConfig,
    ) -> Self {
        let ChatConfig { topic, mods, bans } = config;

        let res = ipfs
            .name_resolve(Some(&mods), false, false)
            .await
            .expect("Invalid Mods Link");

        let mods = ipfs_dag_get_node_async(&ipfs, &res.path)
            .await
            .expect("Invalid Moderators Node");

        let res = ipfs
            .name_resolve(Some(&bans), false, false)
            .await
            .expect("Invalid Mods Link");

        let bans = ipfs_dag_get_node_async(&ipfs, &res.path)
            .await
            .expect("Invalid Moderators Node");

        Self {
            ipfs,

            archive_tx,

            trusted: HashMap::with_capacity(100),

            ban_cache: HashSet::with_capacity(100),
            ban_count: 0,

            topic,

            bans,
            mods,
        }
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

        if self.ban_count > 0 {
            println!("Updating Banned List with {} New Users 👍", self.ban_count);

            if let Err(e) = update_ipns(&self.ipfs, &BANS_KEY, &self.bans).await {
                eprintln!("❗ IPNS Update Failed. {}", e);
            }
        }

        println!("❌ Chat System Offline");
    }

    async fn on_pubsub_message(&mut self, msg: PubsubSubResponse) {
        let from = match msg.from {
            Some(from) => from,
            None => return,
        };

        if self.ban_cache.contains(&from) {
            return;
        }

        let data = match msg.data {
            Some(data) => data,
            None => return,
        };

        let msg: Message = match serde_json::from_slice(&data) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("❗ Deserialization failed. {}", e);
                return;
            }
        };

        let value = self.trusted.get(&from);

        if value.is_none() || value.unwrap().0/*origin*/ != msg.origin.link {
            return self.get_origin(from.to_owned(), msg).await;
        }

        let address = &value.unwrap().1;

        match msg.msg_type {
            MessageType::Unsigned(unmsg) => self.mint_and_archive(unmsg).await,
            MessageType::Ban(ban) => {
                if self.mods.mods.contains(address) {
                    self.ban_cache.insert(ban.peer_id);
                    self.bans.banned.insert(ban.address);
                    self.ban_count += 1;
                }
            }
        }
    }

    async fn get_origin(&mut self, from: String, msg: Message) {
        let sign_msg: SignedMessage =
            match ipfs_dag_get_node_async(&self.ipfs, &msg.origin.link.to_string()).await {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("❗ IPFS: dag get failed {}", e);
                    return;
                }
            };

        if from != sign_msg.data.peer_id {
            return;
        }

        if !sign_msg.verify() {
            return;
        }

        if self.bans.banned.contains(&sign_msg.address) {
            self.ban_cache.insert(from.to_owned());
            return;
        }

        self.trusted
            .insert(from.to_owned(), (msg.origin.link, sign_msg.address));

        self.process_msg(&sign_msg.address, msg).await;
    }

    async fn process_msg(&mut self, address: &Address, msg: Message) {
        match msg.msg_type {
            MessageType::Unsigned(unmsg) => self.mint_and_archive(unmsg).await,
            MessageType::Ban(ban) => self.update_bans(address, ban),
        }
    }

    async fn mint_and_archive(&mut self, msg: UnsignedMessage) {
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

    fn update_bans(&mut self, address: &Address, ban: Ban) {
        if !self.mods.mods.contains(address) {
            return;
        }

        self.ban_cache.insert(ban.peer_id);

        self.bans.banned.insert(ban.address);

        self.ban_count += 1;
    }
}
