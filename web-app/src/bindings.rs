use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use js_sys::{Function, Uint8Array};

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "testMedia")]
    pub fn test_media();

    #[wasm_bindgen(js_name = "cat")]
    pub fn ipfs_cat(path: &str) -> Uint8Array;

    #[wasm_bindgen(js_name = "subscribe")]
    pub fn subscribe(topic: JsValue, pubsub_callback: &Function);

    #[wasm_bindgen(js_name = "publish")]
    pub fn publish(topic: JsValue, message: JsValue);

    #[wasm_bindgen(js_name = "unsubscribe")]
    pub fn unsubscribe(topic: JsValue);

    #[wasm_bindgen(js_name = "registerPlaylistCallback")]
    pub fn register_playlist_callback(playlist_callback: &Function);

    #[wasm_bindgen(js_name = "unregisterPlaylistCallback")]
    pub fn unregister_playlist_callback();

    #[wasm_bindgen(js_name = "initHLS")]
    pub fn init_hls();

    #[wasm_bindgen(js_name = "attachMedia")]
    pub fn hls_attach_media();

    #[wasm_bindgen(js_name = "loadSource")]
    pub fn hls_load_master_playlist();

    #[wasm_bindgen(js_name = "startLoad")]
    pub fn hls_start_load();

    #[wasm_bindgen(js_name = "destroy")]
    pub fn hls_destroy();
}
