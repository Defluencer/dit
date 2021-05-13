use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::components::chat::message::{MessageData, UIMessage};
use crate::utils::ipfs::{IpfsService, PubsubSubResponse};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use cid::Cid;

use linked_data::chat::{Address, Content, SignedMessage, UnsignedMessage};
use linked_data::moderation::{Bans, Moderators};
use linked_data::{Message, MessageType};

use reqwest::Error;

use blockies::Ethereum;

pub struct Display {
    link: ComponentLink<Self>,

    ipfs: IpfsService,
    img_gen: Ethereum,

    /// Signed Message Cid Mapped to address, peer id and name
    trusted_identities: HashMap<Cid, ([u8; 20], String, String)>,

    bans: Option<Bans>,
    mods: Option<Moderators>,

    /// Set of banned peer IDs.
    ban_cache: HashSet<String>,

    /// Peer Id with Messages
    msg_buffer: Vec<(String, Message)>,

    next_id: usize,
    chat_messages: VecDeque<MessageData>,

    drop_sig: Rc<AtomicBool>,
}

pub enum Msg {
    PubSub(Result<PubsubSubResponse, std::io::Error>),
    Origin((Cid, Result<SignedMessage, Error>)),
    BanList(Result<(Cid, Bans), Error>),
    ModList(Result<(Cid, Moderators), Error>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub topic: Rc<str>,
    pub ban_list: Rc<str>,
    pub mod_list: Rc<str>,
}

impl Component for Display {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props {
            ipfs,
            topic,
            ban_list,
            mod_list,
        } = props;

        let client = ipfs.clone();
        let cb = link.callback(Msg::PubSub);
        let sub_topic = topic.to_string();

        let drop_sig = Rc::from(AtomicBool::new(false));
        let sig = drop_sig.clone();

        spawn_local(async move { client.pubsub_sub(sub_topic, cb, sig).await });

        let cb = link.callback_once(Msg::BanList);
        let client = ipfs.clone();
        let ipns = ban_list.to_string();

        spawn_local(async move { cb.emit(client.resolve_and_dag_get(ipns).await) });

        let cb = link.callback_once(Msg::ModList);
        let client = ipfs.clone();
        let ipns = mod_list.to_string();

        spawn_local(async move { cb.emit(client.resolve_and_dag_get(ipns).await) });

        //https://github.com/ethereum/blockies
        //https://docs.rs/blockies/0.3.0/blockies/struct.Ethereum.html
        let img_gen = Ethereum {
            size: 8,
            scale: 4,
            color: None,
            background_color: None,
            spot_color: None,
        };

        Self {
            link,

            ipfs,
            img_gen,

            trusted_identities: HashMap::with_capacity(100),

            bans: None,
            mods: None,

            ban_cache: HashSet::with_capacity(100),

            msg_buffer: Vec::with_capacity(10),

            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,

            drop_sig,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub(result) => self.on_pubsub_update(result),
            Msg::Origin((cid, result)) => self.on_signed_msg(cid, result),
            Msg::BanList(result) => self.on_ban_list_resolved(result),
            Msg::ModList(result) => self.on_mod_list_resolved(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
        <div class="chat_display">
        {
        for self.chat_messages.iter().map(|cm| html! {
            <UIMessage key=cm.id.to_string() message_data=cm />
        })
        }
        </div>
        }
    }

    fn destroy(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping Live Chat");

        self.drop_sig.store(true, Ordering::Relaxed);
    }
}

impl Display {
    /// Callback when GossipSub receive a message.
    fn on_pubsub_update(&mut self, result: Result<PubsubSubResponse, std::io::Error>) -> bool {
        let res = match result {
            Ok(res) => res,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message Received");

        let PubsubSubResponse { from, data } = res;

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        if self.ban_cache.contains(&from) {
            return false;
        }

        let msg: Message = match serde_json::from_slice(&data) {
            Ok(msg) => msg,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        self.process_msg(from, msg)
    }

    fn process_msg(&mut self, from: String, msg: Message) -> bool {
        if !self.trusted_identities.contains_key(&msg.origin.link) {
            self.get_origin(from, msg);
            return false;
        }

        let (addrs, peer_id, name) = &self.trusted_identities[&msg.origin.link];

        if from != *peer_id {
            return false;
        }

        match msg.msg_type {
            MessageType::Unsigned(msg) => {
                //self.update_display(addrs, name, &msg);
                #[cfg(debug_assertions)]
                ConsoleService::info(&format!("Message => {}", &msg.message));
                let mut data = Vec::new();
                self.img_gen
                    .create_icon(&mut data, addrs)
                    .expect("Invalid Blocky");
                let msg_data = MessageData::new(self.next_id, &data, &name, &msg.message);
                self.chat_messages.push_back(msg_data);
                if self.chat_messages.len() >= 10 {
                    self.chat_messages.pop_front();
                }
                self.next_id += 1;
                return true;
            }
            MessageType::Ban(ban) => {
                if let Some(mod_list) = &self.mods {
                    if mod_list.mods.contains(addrs) {
                        self.ban_cache.insert(ban.peer_id);
                    }
                }
            }
        }

        false
    }

    fn get_origin(&mut self, from: String, msg: Message) {
        let cb = self.link.callback_once(Msg::Origin);
        let client = self.ipfs.clone();
        let cid = msg.origin.link;

        self.msg_buffer.push((from, msg));

        spawn_local(
            async move { cb.emit((cid, client.dag_get(cid, Option::<String>::None).await)) },
        );
    }

    fn update_display(&mut self, addrs: &Address, name: &str, msg: &UnsignedMessage) {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Message => {}", &msg.message));

        let mut data = Vec::new();

        self.img_gen
            .create_icon(&mut data, addrs)
            .expect("Invalid Blocky");

        let msg_data = MessageData::new(self.next_id, &data, &name, &msg.message);

        self.chat_messages.push_back(msg_data);

        if self.chat_messages.len() >= 10 {
            self.chat_messages.pop_front();
        }

        self.next_id += 1;
    }

    /// Callback when IPFS dag get signed message node.
    fn on_signed_msg(&mut self, cid: Cid, response: Result<SignedMessage, Error>) -> bool {
        let sign_msg = match response {
            Ok(m) => m,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Signed Message Received");

        let verified = sign_msg.verify();

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Verifiable => {}", verified));

        let SignedMessage {
            address,
            data,
            signature: _,
        } = sign_msg;

        let Content { peer_id, name } = data;

        let mut update = false;

        let mut i = self.msg_buffer.len();
        while i != 0 {
            let (_, msg) = &self.msg_buffer[i - 1];

            let (from, msg) = if cid != msg.origin.link {
                continue;
            } else {
                let msg = self.msg_buffer.swap_remove(i - 1);

                i -= 1;

                msg
            };

            if from == peer_id && verified {
                match msg.msg_type {
                    MessageType::Unsigned(msg) => {
                        self.update_display(&address, &name, &msg);
                        update = true;
                    }
                    MessageType::Ban(ban) => {
                        if let Some(mod_list) = &self.mods {
                            if mod_list.mods.contains(&address) {
                                self.ban_cache.insert(ban.peer_id);
                            }
                        }
                    }
                }
            }
        }

        if verified {
            if let Some(ban_list) = &self.bans {
                if ban_list.banned.contains(&address) {
                    self.ban_cache.insert(peer_id.to_owned());
                }
            }

            self.trusted_identities
                .insert(cid, (address, peer_id, name));
        }

        update
    }

    /// Callback when IPFS dag get ban list node.
    fn on_ban_list_resolved(&mut self, result: Result<(Cid, Bans), Error>) -> bool {
        let bans = match result {
            Ok((_, bans)) => bans,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Chat Ban List Received");

        self.bans = Some(bans);

        false
    }

    /// Callback when IPFS dag get mod list node.
    fn on_mod_list_resolved(&mut self, result: Result<(Cid, Moderators), Error>) -> bool {
        let mods = match result {
            Ok((_, mods)) => mods,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Chat Moderator List Received");

        self.mods = Some(mods);

        false
    }
}
