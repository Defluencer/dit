use std::collections::VecDeque;
use std::rc::Rc;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::components::chat::message::{MessageData, UIMessage};
use crate::utils::ipfs::{IpfsService, PubsubSubResponse};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use cid::Cid;

use linked_data::chat::{ChatId, UnsignedMessage};
use linked_data::messaging::{Message, MessageType};
use linked_data::moderation::{Ban, Bans, ChatModerationCache, Moderators};
use linked_data::signature::SignedMessage;
use linked_data::PeerId;

use reqwest::Error;

use blockies::Ethereum;

pub struct Display {
    link: ComponentLink<Self>,

    ipfs: IpfsService,
    img_gen: Ethereum,

    mod_db: ChatModerationCache,

    bans: Option<Bans>,
    mods: Option<Moderators>,

    next_id: usize,
    chat_messages: VecDeque<MessageData>,

    drop_sig: Rc<AtomicBool>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    PubSub(Result<PubsubSubResponse, std::io::Error>),
    Origin((PeerId, Message, Result<SignedMessage<ChatId>, Error>)),
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

            mod_db: ChatModerationCache::new(100, 100),

            bans: None,
            mods: None,

            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,

            drop_sig,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub(result) => self.on_pubsub_update(result),
            Msg::Origin((peer, msg, result)) => self.on_signed_msg(peer, msg, result),
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

        if self.mod_db.is_banned(&from) {
            return false;
        }

        let msg: Message = match serde_json::from_slice(&data) {
            Ok(msg) => msg,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if !self.mod_db.is_verified(&from, &msg.origin.link) {
            self.get_origin(from, msg);
            return false;
        }

        self.process_msg(from, msg)
    }

    fn get_origin(&mut self, from: String, msg: Message) {
        let cb = self.link.callback_once(Msg::Origin);
        let client = self.ipfs.clone();
        let cid = msg.origin.link;

        spawn_local(async move {
            cb.emit((from, msg, client.dag_get(cid, Option::<String>::None).await))
        });
    }

    /// Callback when IPFS dag get return signed message node.
    fn on_signed_msg(
        &mut self,
        peer: String,
        msg: Message,
        response: Result<SignedMessage<ChatId>, Error>,
    ) -> bool {
        let sign_msg = match response {
            Ok(m) => m,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Signed Message Received");

        let trusted = sign_msg.verify();

        self.mod_db.add_peer(
            &sign_msg.data.peer,
            msg.origin.link,
            sign_msg.address,
            Some(sign_msg.data.name),
        );

        if peer != sign_msg.data.peer {
            self.mod_db.ban_peer(&peer);
            return false;
        }

        if !trusted {
            self.mod_db.ban_peer(&peer);

            #[cfg(debug_assertions)]
            ConsoleService::info("Verifiable => false");

            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Verifiable => true");

        if let Some(bans) = self.bans.as_ref() {
            if bans.banned.contains(&sign_msg.address) {
                self.mod_db.ban_peer(&peer);
                return false;
            }
        }

        self.process_msg(peer, msg)
    }

    fn process_msg(&mut self, peer: PeerId, msg: Message) -> bool {
        match msg.msg_type {
            MessageType::Unsigned(msg) => self.update_display(&peer, &msg),
            MessageType::Ban(ban) => self.update_bans(&peer, ban),
            MessageType::Mod(_) => false,
        }
    }

    fn update_display(&mut self, peer: &str, msg: &UnsignedMessage) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Message => {}", &msg.message));

        let address = self.mod_db.get_address(peer).unwrap();
        let name = self.mod_db.get_name(peer).unwrap();

        let mut data = Vec::new();

        self.img_gen
            .create_icon(&mut data, address)
            .expect("Invalid Blocky");

        let msg_data = MessageData::new(self.next_id, &data, &name, &msg.message);

        self.chat_messages.push_back(msg_data);

        if self.chat_messages.len() >= 10 {
            self.chat_messages.pop_front();
        }

        self.next_id += 1;

        true
    }

    fn update_bans(&mut self, peer: &str, ban: Ban) -> bool {
        let mods = match self.mods.as_ref() {
            Some(mods) => mods,
            None => return false,
        };

        let address = self.mod_db.get_address(peer).unwrap();

        if !mods.mods.contains(address) {
            return false;
        }

        self.mod_db.ban_peer(&ban.peer_id);
        self.bans.as_mut().unwrap().banned.insert(ban.address);

        false
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
