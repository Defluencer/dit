use std::sync::{Arc, RwLock};

use crate::playlists::Playlists;

use wasm_bindgen::prelude::{wasm_bindgen, Closure, JsValue};
use wasm_bindgen::JsCast;

use js_sys::Function;

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initLibs")]
    fn init_libs(topic: JsValue, pubsub_callback: &Function, playlist_callback: &Function);

    #[wasm_bindgen(js_name = "startVideo")]
    pub fn start_video();
}

pub fn init(topic: &str) {
    let arc = Arc::new(RwLock::new(Playlists::new()));

    let arc_clone = arc.clone();

    let playlist_closure = Closure::wrap(Box::new(move |url| match arc_clone.read() {
        Ok(playlist) => playlist.get_playlist(url),
        Err(_) => String::from("Lock Poisoned"),
    }) as Box<dyn Fn(String) -> String>);

    let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
        if let Ok(mut playlist) = arc.write() {
            playlist.pubsub_message(from, data);
        }
    }) as Box<dyn Fn(String, Vec<u8>)>);

    init_libs(
        topic.into(),
        pubsub_closure.into_js_value().unchecked_ref(),
        playlist_closure.into_js_value().unchecked_ref(),
    );
}
