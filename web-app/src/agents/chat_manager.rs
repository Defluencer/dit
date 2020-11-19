use crate::bindings;

//use std::collections::HashSet;
//use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use yew::services::ConsoleService;
//use yew::worker::{Agent, AgentLink, HandlerId, Public};
use yew::Callback;

const TOPIC: &str = "livelikechat";

pub fn load_live_chat(cb: Callback<String>) {
    let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
        let msg = match pubsub_message(from, data) {
            Some(msg) => msg,
            None => return,
        };

        cb.emit(msg);
    }) as Box<dyn Fn(String, Vec<u8>)>);

    bindings::subscribe(TOPIC.into(), pubsub_closure.into_js_value().unchecked_ref());
}

pub fn unload_live_chat() {
    bindings::unsubscribe(TOPIC.into());
}

pub fn send_chat(msg: String) {
    bindings::publish(TOPIC.into(), msg.into());
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
