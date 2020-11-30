use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use js_sys::{Function, Uint8Array};

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "loadStream")]
    pub fn load_stream(topic: &str);

    #[wasm_bindgen(js_name = "loadVideo")]
    pub fn load_video(cid: &str);

    #[wasm_bindgen(js_name = "cat")]
    pub fn ipfs_cat(path: &str) -> Uint8Array;

    #[wasm_bindgen(js_name = "subscribe")]
    pub fn subscribe(topic: JsValue, pubsub_callback: &Function);

    #[wasm_bindgen(js_name = "publish")]
    pub fn publish(topic: JsValue, message: JsValue);

    #[wasm_bindgen(js_name = "unsubscribe")]
    pub fn unsubscribe(topic: JsValue);
}
