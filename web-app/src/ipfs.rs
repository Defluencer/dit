#![allow(dead_code)]

use yew::services::ConsoleService;

use wasm_bindgen::prelude::{wasm_bindgen, Closure, JsValue};
use wasm_bindgen::JsCast;

use js_sys::Function;

#[wasm_bindgen(module = "/ipfs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initIPFS")]
    fn init_ipfs(topic: JsValue, callback: &Function);
}

fn pubsub_message(from: String, data: Vec<u8>) {
    let string = String::from_utf8_lossy(&data);

    ConsoleService::info(&format!("from={} data={}", from, string));
}

pub fn init(topic: &str) {
    let closure = Closure::wrap(Box::new(pubsub_message) as Box<dyn FnMut(String, Vec<u8>)>);

    init_ipfs(topic.into(), closure.into_js_value().unchecked_ref());
}
