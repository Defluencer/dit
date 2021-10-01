use std::collections::VecDeque;
use std::rc::Rc;
use std::str;

use crate::components::chat::message::{MessageData, UIMessage};
use crate::components::IPFSPubSubError;
use crate::utils::IpfsService;

use futures::future::AbortHandle;

use wasm_bindgen_futures::spawn_local;
use web_sys::Element;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::chat::{ChatId, Message, MessageType, UnsignedMessage};
use linked_data::live::Live;
use linked_data::moderation::{Ban, Bans, ChatModerationCache, Moderators};
use linked_data::signature::SignedMessage;
use linked_data::PeerId;

use blockies::Ethereum;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Display {
    props: Props,

    error: bool,

    msg_cb: Callback<(PeerId, Message, Result<SignedMessage<ChatId>>)>,

    pubsub_cb: Callback<Result<(String, Vec<u8>)>>,
    handle: AbortHandle,

    img_gen: Ethereum,

    mod_db: ChatModerationCache,

    chat_element: Option<Element>,

    next_id: usize,
    chat_messages: VecDeque<MessageData>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    PubSub(Result<(String, Vec<u8>)>),
    Origin((PeerId, Message, Result<SignedMessage<ChatId>>)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub live: Rc<Live>,
    pub mods: Rc<Moderators>,
    pub bans: Rc<Bans>,
}

impl Component for Display {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        //https://github.com/ethereum/blockies
        //https://docs.rs/blockies/0.3.0/blockies/struct.Ethereum.html
        let img_gen = Ethereum {
            size: 8,
            scale: 4,
            color: None,
            background_color: None,
            spot_color: None,
        };

        let pubsub_cb = link.callback(Msg::PubSub);
        let (handle, regis) = AbortHandle::new_pair();

        if !props.live.chat_topic.is_empty() {
            spawn_local({
                let ipfs = props.ipfs.clone();
                let sub_topic = props.live.chat_topic.clone();
                let cb = pubsub_cb.clone();

                async move { ipfs.pubsub_sub(sub_topic, cb, regis).await }
            });
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Chat Display Created");

        Self {
            props,

            error: false,

            msg_cb: link.callback(Msg::Origin),

            pubsub_cb,
            handle,

            img_gen,

            mod_db: ChatModerationCache::new(100, 100),

            chat_element: None,

            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub(result) => self.on_pubsub_update(result),
            Msg::Origin((peer, msg, result)) => self.on_signed_msg(peer, msg, result),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&self.props.live, &props.live) {
            self.handle.abort();

            self.props = props;

            if !self.props.live.chat_topic.is_empty() {
                let (handle, regis) = AbortHandle::new_pair();

                self.handle = handle;

                spawn_local({
                    let ipfs = self.props.ipfs.clone();
                    let sub_topic = self.props.live.chat_topic.clone();
                    let cb = self.pubsub_cb.clone();

                    async move { ipfs.pubsub_sub(sub_topic, cb, regis).await }
                });
            }

            #[cfg(debug_assertions)]
            ConsoleService::info("Chat Display Changed");
        }

        false
    }

    fn view(&self) -> Html {
        if self.error {
            return html! { <IPFSPubSubError /> };
        }

        html! {
            <div id="chat_display" class="box" style="overflow-y: scroll;height: 60vh;scroll-behavior: smooth;" >
            {
                for self.chat_messages.iter().map(|cm| html! {
                    <UIMessage key=cm.id.to_string() message_data=cm.clone() />
                })
            }
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if !first_render {
            if let Some(element) = self.chat_element.as_mut() {
                element.set_scroll_top(element.scroll_height());
            }

            return;
        }

        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Window Object");
                return;
            }
        };

        let document = match window.document() {
            Some(document) => document,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Document Object");
                return;
            }
        };

        let element = match document.get_element_by_id("chat_display") {
            Some(document) => document,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Element by Id");
                return;
            }
        };

        self.chat_element = Some(element);
    }

    fn destroy(&mut self) {
        self.handle.abort()
    }
}

impl Display {
    /// Callback when GossipSub receive a message.
    fn on_pubsub_update(&mut self, result: Result<(String, Vec<u8>)>) -> bool {
        let res = match result {
            Ok(res) => res,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                self.error = true;
                return true;
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
            let cb = self.msg_cb.clone();
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
            MessageType::Mod(_) => false, //TODO
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
