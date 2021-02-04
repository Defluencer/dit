use crate::utils::bindings::{ipfs_publish, ipfs_subscribe, ipfs_unsubscribe};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use yew::services::ConsoleService;
use yew::Callback;

pub struct LiveChatManager {
    topic: String,
}

impl LiveChatManager {
    pub fn new(topic: String, cb: Callback<(String, String)>) -> Self {
        load_live_chat(&topic, cb);

        Self { topic }
    }

    pub fn send_chat(&self, msg: &str) {
        ipfs_publish(&self.topic, msg);
    }
}

impl Drop for LiveChatManager {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping LiveChatManager");

        ipfs_unsubscribe(&self.topic);
    }
}

fn load_live_chat(topic: &str, cb: Callback<(String, String)>) {
    let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
        let msg = match pubsub_message(from, data) {
            Some(msg) => msg,
            None => return,
        };

        cb.emit(msg);
    }) as Box<dyn Fn(String, Vec<u8>)>);

    ipfs_subscribe(topic, pubsub_closure.into_js_value().unchecked_ref());
}

fn pubsub_message(from: String, data: Vec<u8>) -> Option<(String, String)> {
    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Sender => {}", from));

    let msg = match String::from_utf8(data) {
        Ok(string) => string,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Invalid UTF-8");

            return None;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Message => {}", msg));

    Some((from, msg))
}
