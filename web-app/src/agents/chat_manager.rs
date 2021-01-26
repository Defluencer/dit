use crate::utils::{ipfs_publish, ipfs_subscribe, ipfs_unsubscribe};

//use std::collections::HashSet;
//use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use yew::services::ConsoleService;
//use yew::worker::{Agent, AgentLink, HandlerId, Public};
use yew::Callback;

use linked_data::LIVE_CHAT_TOPIC;

pub fn load_live_chat(cb: Callback<String>) {
    let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
        let msg = match pubsub_message(from, data) {
            Some(msg) => msg,
            None => return,
        };

        cb.emit(msg);
    }) as Box<dyn Fn(String, Vec<u8>)>);

    ipfs_subscribe(
        LIVE_CHAT_TOPIC.into(),
        pubsub_closure.into_js_value().unchecked_ref(),
    );
}

pub fn unload_live_chat() {
    ipfs_unsubscribe(LIVE_CHAT_TOPIC.into());
}

pub fn send_chat(msg: String) {
    ipfs_publish(LIVE_CHAT_TOPIC.into(), msg.into());
}

fn pubsub_message(from: String, data: Vec<u8>) -> Option<String> {
    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Sender => {}", from));

    //TODO process msg only if sender is not ban/blocked

    let msg = match String::from_utf8(data) {
        Ok(string) => string,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Message Invalid UTF-8");

            return None;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Message => {}", msg));

    Some(msg)
}
