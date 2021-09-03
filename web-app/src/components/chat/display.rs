use std::collections::VecDeque;
use std::rc::Rc;
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::components::chat::message::{MessageData, UIMessage};
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::Beacon;
use linked_data::chat::{ChatId, Message, MessageType, UnsignedMessage};
use linked_data::moderation::{Ban, Bans, ChatModerationCache, Moderators};
use linked_data::signature::SignedMessage;
use linked_data::PeerId;

use blockies::Ethereum;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Display {
    props: Props,
    link: ComponentLink<Self>,

    img_gen: Ethereum,

    mod_db: ChatModerationCache,

    next_id: usize,
    chat_messages: VecDeque<MessageData>,

    drop_sig: Rc<AtomicBool>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    PubSub(Result<(String, Vec<u8>)>),
    Origin((PeerId, Message, Result<SignedMessage<ChatId>>)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub beacon: Rc<Beacon>,
    pub mods: Rc<Moderators>,
    pub bans: Rc<Bans>,
}

impl Component for Display {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let drop_sig = Rc::from(AtomicBool::new(false));

        spawn_local({
            let ipfs = props.ipfs.clone();
            let sub_topic = props.beacon.topics.chat.clone();
            let cb = link.callback(Msg::PubSub);
            let sig = drop_sig.clone();

            async move { ipfs.pubsub_sub(sub_topic, cb, sig).await }
        });

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
            props,
            link,

            img_gen,

            mod_db: ChatModerationCache::new(100, 100),

            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,

            drop_sig,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub(result) => self.on_pubsub_update(result),
            Msg::Origin((peer, msg, result)) => self.on_signed_msg(peer, msg, result),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&self.props.beacon, &props.beacon)
            || !Rc::ptr_eq(&self.props.mods, &props.mods)
            || !Rc::ptr_eq(&self.props.bans, &props.bans)
        {
            self.props = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
        <div class="chat_display">
        {
        for self.chat_messages.iter().map(|cm| html! {
            <UIMessage key=cm.id.to_string() message_data=cm.clone() />
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
    fn on_pubsub_update(&mut self, result: Result<(String, Vec<u8>)>) -> bool {
        let res = match result {
            Ok(res) => res,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message Received");

        let (from, data) = res;

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

    fn get_origin(&self, from: String, msg: Message) {
        spawn_local({
            let cb = self.link.callback_once(Msg::Origin);
            let ipfs = self.props.ipfs.clone();
            let cid = msg.origin.link;

            async move { cb.emit((from, msg, ipfs.dag_get(cid, Option::<String>::None).await)) }
        });
    }

    /// Callback when IPFS dag get return signed message node.
    fn on_signed_msg(
        &mut self,
        peer: String,
        msg: Message,
        response: Result<SignedMessage<ChatId>>,
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

        if self.props.bans.banned.contains(&sign_msg.address) {
            self.mod_db.ban_peer(&peer);
            return false;
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

        let address = match self.mod_db.get_address(peer) {
            Some(addrs) => addrs,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Address");
                return false;
            }
        };

        let name = match self.mod_db.get_name(peer) {
            Some(name) => name,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Name");
                return false;
            }
        };

        let mut data = Vec::new();

        if let Err(e) = self.img_gen.create_icon(&mut data, address) {
            ConsoleService::error(&format!("{:?}", e));
        }

        let msg_data = MessageData::new(self.next_id, &data, name, &msg.message);

        self.chat_messages.push_back(msg_data);

        if self.chat_messages.len() >= 10 {
            self.chat_messages.pop_front();
        }

        self.next_id += 1;

        true
    }

    fn update_bans(&mut self, peer: &str, ban: Ban) -> bool {
        let address = match self.mod_db.get_address(peer) {
            Some(addrs) => addrs,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Address");
                return false;
            }
        };

        if !self.props.mods.mods.contains(address) {
            return false;
        }

        self.mod_db.ban_peer(&ban.peer_id);

        false
    }
}
