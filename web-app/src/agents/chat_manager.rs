use crate::bindings;

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use yew::services::ConsoleService;
use yew::worker::{Agent, AgentLink, HandlerId, Public};

const TOPIC: &str = "livelikechat";

//use serde::{Deserialize, Serialize};

/* #[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    ChatMsg(String),
} */

pub struct ChatManager {
    _link: Arc<RwLock<AgentLink<ChatManager>>>,
    subscribers: Arc<RwLock<HashSet<HandlerId>>>,
}

impl Agent for ChatManager {
    type Reach = Public<Self>;
    type Message = ();
    type Input = String;
    type Output = String;

    fn create(link: AgentLink<Self>) -> Self {
        let subscribers = Arc::new(RwLock::new(HashSet::new()));
        let link = Arc::new(RwLock::new(link));

        let subscribers_clone = subscribers.clone();
        let link_clone = link.clone();

        let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
            let msg = match pubsub_message(from, data) {
                Some(msg) => msg,
                None => return,
            };

            {
                let subscribers = subscribers_clone.read().expect("On PubSub");
                let link = link_clone.read().expect("On PubSub");

                for sub in subscribers.iter() {
                    link.respond(*sub, msg.clone());
                }
            }
        }) as Box<dyn Fn(String, Vec<u8>)>);

        bindings::subscribe(TOPIC.into(), pubsub_closure.into_js_value().unchecked_ref());

        Self {
            _link: link,
            subscribers,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        bindings::publish(TOPIC.into(), msg.into());
    }

    fn connected(&mut self, id: HandlerId) {
        let mut subscribers = match self.subscribers.write() {
            Ok(sub) => sub,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::error("RwLock Poisoned");
                return;
            }
        };

        subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        let mut subscribers = match self.subscribers.write() {
            Ok(sub) => sub,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::error("RwLock Poisoned");
                return;
            }
        };

        subscribers.remove(&id);
    }
}

fn pubsub_message(from: String, data: Vec<u8>) -> Option<String> {
    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Sender => {}", from));

    //TODO process msg only if sender is not ban/blocked
    /* if from != STREAMER_PEER_ID {
        #[cfg(debug_assertions)]
        ConsoleService::warn("Unauthorized Sender");

        return None;
    } */

    let msg = match String::from_utf8(data) {
        Ok(string) => string,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Message Invalid UTF-8");

            return None;
        }
    };

    /* let video_cid = match Cid::try_from(data_utf8) {
        Ok(cid) => cid,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Message Invalid CID");

            return None;
        }
    }; */

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Message => {}", msg));

    Some(msg)
}
