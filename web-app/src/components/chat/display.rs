use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::str;

use crate::components::chat::message::{MessageData, UIMessage};
use crate::utils::bindings::{ipfs_dag_get, ipfs_subscribe, ipfs_unsubscribe};

use wasm_bindgen_futures::spawn_local;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use cid::Cid;

use linked_data::chat::{SignedMessage, UnsignedMessage};

const SIGN_MSG_KEY: &str = "signed_message";

pub struct Display {
    link: ComponentLink<Self>,

    topic: String,
    _pubsub_closure: Closure<dyn Fn(String, Vec<u8>)>,

    trusted: HashMap<(String, Cid), String>,

    next_id: usize,
    chat_messages: VecDeque<MessageData>,
}

pub enum Msg {
    PubSub((String, Vec<u8>)),
    Origin((String, UnsignedMessage, SignedMessage)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub topic: String,
}

impl Component for Display {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let topic = props.topic;

        let cb = link.callback(Msg::PubSub);
        let _pubsub_closure =
            Closure::wrap(
                Box::new(move |from: String, data: Vec<u8>| cb.emit((from, data)))
                    as Box<dyn Fn(String, Vec<u8>)>,
            );

        ipfs_subscribe(&topic, _pubsub_closure.as_ref().unchecked_ref());

        Self {
            link,
            topic,
            _pubsub_closure,
            trusted: HashMap::with_capacity(100),

            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub((from, data)) => self.on_pubsub_update(from, data),
            Msg::Origin((from, msg, sign_msg)) => self.on_signed_msg(from, msg, sign_msg),
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

        ipfs_unsubscribe(&self.topic);
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

        if let Some(name) = self.trusted.get(&(from, msg.origin.link)) {
            return self.display_msg(name, &msg.message);
        }

        let cb = self.link.callback_once(Msg::Origin);
        spawn_local(get_sign_msg_async(from, msg, cb));

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
        from: String,
        msg: UnsignedMessage,
        sign_msg: SignedMessage,
    ) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Signed Message => {:#?}", sign_msg));

        if from != sign_msg.data.peer_id {
            return false;
        }

        if !sign_msg.verify() {
            return false;
        }

        self.trusted
            .insert((from, msg.origin.link), sign_msg.data.name);

        self.display_msg(&sign_msg.data.name, &msg.message)
    }
}

async fn get_sign_msg_async(
    from: String,
    msg: UnsignedMessage,
    cb: Callback<(String, UnsignedMessage, SignedMessage)>,
) {
    let node = match ipfs_dag_get(&msg.origin.link.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let sign_msg: SignedMessage = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    cb.emit((from, msg, sign_msg));
}
