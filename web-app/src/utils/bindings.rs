use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use js_sys::Function;

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "subscribe")]
    pub fn ipfs_subscribe(topic: JsValue, pubsub_callback: &Function);

    #[wasm_bindgen(js_name = "publish")]
    pub fn ipfs_publish(topic: JsValue, message: JsValue);

    #[wasm_bindgen(js_name = "unsubscribe")]
    pub fn ipfs_unsubscribe(topic: JsValue);

    #[wasm_bindgen(js_name = "dagGet", catch)]
    pub async fn ipfs_dag_get(cid: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "cat", catch)]
    pub async fn ipfs_cat(path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "waitUntil")]
    pub async fn wait_until(fn_bool: &Function);

}
