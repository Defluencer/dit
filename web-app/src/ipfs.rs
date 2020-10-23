#![allow(dead_code)]

use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

use wasm_bindgen::JsCast;

#[derive(Serialize, Deserialize)]
struct Message {
    pub from: String,
    pub data: String,
}

#[wasm_bindgen(module = "/ipfs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initIPFS")]
    fn init_ipfs(topic: JsValue, callback: &js_sys::Function);
}

fn pubsub_message(data: JsValue) {
    let _msg: Message = match data.into_serde() {
        Ok(msg) => msg,
        Err(_) => {
            //TODO print error
            return;
        }
    };

    web_sys::console::log_1(&"PubSub Message Received".into());
}

pub fn init() {
    let callback = Closure::wrap(Box::new(pubsub_message) as Box<dyn FnMut(JsValue)>);

    let topic = "livelike";

    init_ipfs(topic.into(), callback.into_js_value().unchecked_ref());
}
