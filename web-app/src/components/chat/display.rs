use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::str;

use crate::components::chat::message::{MessageData, UIMessage};
use crate::utils::ipfs::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use cid::Cid;

use linked_data::chat::{SignedMessage, UnsignedMessage};

pub struct Display {
    link: ComponentLink<Self>,

    ipfs: IpfsService,
    topic: Rc<str>,

    trusted_identities: HashMap<Cid, (String, String)>,

    next_id: usize,
    chat_messages: VecDeque<MessageData>,
}

pub enum Msg {
    PubSub((String, Vec<u8>)),
    Origin(Result<SignedMessage, ipfs_api::response::Error>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub topic: Rc<str>,
}

impl Component for Display {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, topic } = props;

        //TODO ipfs_subscribe(&topic, _pubsub_closure.as_ref().unchecked_ref());

        Self {
            link,

            ipfs,
            topic,
            trusted_identities: HashMap::with_capacity(100),

            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub((from, data)) => self.on_pubsub_update(from, data),
            Msg::Origin(result) => self.on_signed_msg(result),
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

        //TODO ipfs_unsubscribe(&self.topic);
    }
}

impl Display {
    /// Callback when GossipSub receive a message.
    fn on_pubsub_update(&mut self, from: String, data: Vec<u8>) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message");

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        let msg: UnsignedMessage = match serde_json::from_slice(&data) {
            Ok(msg) => msg,
            Err(_) => return false,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Unsigned Message => {:#?}", msg));

        if !self.is_allowed(&from, &msg.origin.link) {
            return false;
        }

        if let Some((peer_id, name)) = self.trusted_identities.get(&msg.origin.link) {
            if *peer_id == from {
                let name = name.clone();

                return self.display_msg(&name, &msg.message);
            }
        }

        let cb = self.link.callback_once(Msg::Origin);
        let client = self.ipfs.clone();
        let cid = msg.origin.link;

        spawn_local(async move { cb.emit(client.dag_get(cid, Option::<String>::None).await) });

        false
    }

    /// Verify identity against white & black lists
    fn is_allowed(&self, _from: &str, _cid: &Cid) -> bool {
        //TODO verify white & black list
        true
        //self.whitelist.whitelist.contains(identity) || !self.blacklist.blacklist.contains(identity)
    }

    fn display_msg(&mut self, name: &str, msg: &str) -> bool {
        let msg_data = MessageData {
            id: self.next_id,
            sender_name: Rc::from(name),
            message: Rc::from(msg),
        };

        self.chat_messages.push_back(msg_data);

        if self.chat_messages.len() >= 10 {
            self.chat_messages.pop_front();
        }

        self.next_id += 1;

        true
    }

    /// Callback when IPFS get signed message dag node.
    fn on_signed_msg(
        &mut self,
        response: Result<SignedMessage, ipfs_api::response::Error>,
    ) -> bool {
        let sign_msg = match response {
            Ok(m) => m,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Signed Message => {:#?}", sign_msg));
        false
        //TODO

        /* if from != sign_msg.data.peer_id {
            return false;
        }

        if !sign_msg.verify() {
            return false;
        }

        self.trusted_identities
            .insert(msg.origin.link, (from, sign_msg.data.name.clone()));

        self.display_msg(&sign_msg.data.name, &msg.message) */
    }
}
