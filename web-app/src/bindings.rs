use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use js_sys::Function;

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "subscribe")]
    pub fn subscribe(topic: JsValue, pubsub_callback: &Function);

    #[wasm_bindgen(js_name = "initHLS")]
    pub fn init_hls(playlist_callback: &Function);

    /* #[wasm_bindgen(js_name = "attachMedia")]
    pub fn attach_media(); */

    /* #[wasm_bindgen(js_name = "loadSource")]
    pub fn load_source(); */

    #[wasm_bindgen(js_name = "startLoad")]
    pub fn start_load();
}
