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

use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;

pub struct Display {
    link: ComponentLink<Self>,

    ipfs: IpfsService,
    //topic: Rc<str>,
    trusted_identities: HashMap<Cid, (String, String)>,

    msg_buffer: Vec<(String, UnsignedMessage)>,

    next_id: usize,
    chat_messages: VecDeque<MessageData>,
}

pub enum Msg {
    PubSub(Result<PubsubSubResponse, Error>),
    Origin((Cid, Result<SignedMessage, Error>)),
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

        let client = ipfs.clone();
        let cb = link.callback(Msg::PubSub);
        let sub_topic = topic.to_string();

        spawn_local(async move { client.pubsub_sub(sub_topic, cb).await });

        Self {
            link,

            ipfs,
            //topic,
            trusted_identities: HashMap::with_capacity(100),

            msg_buffer: Vec::with_capacity(10),

            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub(result) => self.on_pubsub_update(result),
            Msg::Origin((cid, result)) => self.on_signed_msg(cid, result),
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

        //ipfs_unsubscribe(&self.topic);
    }
}

impl Display {
    /// Callback when GossipSub receive a message.
    fn on_pubsub_update(&mut self, result: Result<PubsubSubResponse, Error>) -> bool {
        let response = match result {
            Ok(res) => res,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        let PubsubSubResponse {
            from,
            data,
            seqno: _,
            topic_ids: _,
            unrecognized: _,
        } = response;

        let (from, data) = match (from, data) {
            (Some(from), Some(data)) => (from, data),
            _ => return false,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message");

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        let msg: UnsignedMessage = match serde_json::from_slice(&data) {
            Ok(msg) => msg,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Unsigned Message => {:#?}", msg));

        if !self.is_allowed(&from, &msg.origin.link) {
            return false;
        }

        match self.trusted_identities.get(&msg.origin.link) {
            Some(value) => {
                if value.0 == from {
                    let msg_data = MessageData {
                        id: self.next_id,
                        sender_name: Rc::from(value.1.clone()),
                        message: Rc::from(msg.message),
                    };

                    self.chat_messages.push_back(msg_data);

                    if self.chat_messages.len() >= 10 {
                        self.chat_messages.pop_front();
                    }

                    self.next_id += 1;

                    return true;
                }
            }
            None => {
                let cb = self.link.callback_once(Msg::Origin);
                let client = self.ipfs.clone();
                let cid = msg.origin.link;

                self.msg_buffer.push((from, msg));

                spawn_local(async move {
                    cb.emit((cid, client.dag_get(cid, Option::<String>::None).await))
                });
            }
        }

        false
    }

    /// Verify identity against white & black lists
    fn is_allowed(&self, _from: &str, _cid: &Cid) -> bool {
        //TODO verify white & black list
        true
        //self.whitelist.whitelist.contains(identity) || !self.blacklist.blacklist.contains(identity)
    }

    /// Callback when IPFS dag get signed message node.
    fn on_signed_msg(
        &mut self,
        cid: Cid,
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

        let verified = sign_msg.verify();

        if verified {
            self.trusted_identities.insert(
                cid,
                (sign_msg.data.peer_id.clone(), sign_msg.data.name.clone()),
            );
        }

        let mut i = self.msg_buffer.len() - 1;
        while i != 0 {
            let (from, msg) = &self.msg_buffer[i];

            if cid != msg.origin.link {
                continue;
            }

            if *from == sign_msg.data.peer_id && verified {
                let msg_data = MessageData {
                    id: self.next_id,
                    sender_name: Rc::from(sign_msg.data.name.clone()),
                    message: Rc::from(msg.message.clone()),
                };

                self.chat_messages.push_back(msg_data);

                if self.chat_messages.len() >= 10 {
                    self.chat_messages.pop_front();
                }

                self.next_id += 1;
            }

            self.msg_buffer.swap_remove(i);

            i -= 1;
        }

        true
    }
}
