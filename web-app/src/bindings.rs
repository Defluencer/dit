use yew::services::ConsoleService;

use wasm_bindgen::prelude::{wasm_bindgen, Closure, JsValue};
use wasm_bindgen::JsCast;

use js_sys::Function;

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initLibs")]
    fn init_libs(
        topic: JsValue,
        pubsub_callback: &Function,
        master_callback: &Function,
        frag_callback: &Function,
    );
}

fn pubsub_message(from: String, data: Vec<u8>) {
    let data_string = String::from_utf8_lossy(&data);

    ConsoleService::info(&format!("from={} data={}", &from, data_string));
}

fn load_master_playlist() -> String {
    String::from("This is a Master Playlist")
}

fn load_fragment_playlist() -> String {
    String::from("This is a Fragment Playlist")
}

pub fn init() {
    let topic = "livelike";

    let pubsub_closure = Closure::wrap(Box::new(pubsub_message) as Box<dyn Fn(String, Vec<u8>)>);

    let master_closure = Closure::wrap(Box::new(load_master_playlist) as Box<dyn Fn() -> String>);

    let frag_closure = Closure::wrap(Box::new(load_fragment_playlist) as Box<dyn Fn() -> String>);

    init_libs(
        topic.into(),
        pubsub_closure.into_js_value().unchecked_ref(),
        master_closure.into_js_value().unchecked_ref(),
        frag_closure.into_js_value().unchecked_ref(),
    );
}
