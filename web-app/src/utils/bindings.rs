use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use js_sys::Function;

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "subscribe")]
    pub fn ipfs_subscribe(topic: &str, pubsub_callback: &Function);

    #[wasm_bindgen(js_name = "publish")]
    pub fn ipfs_publish(topic: &str, message: &str);

    #[wasm_bindgen(js_name = "unsubscribe")]
    pub fn ipfs_unsubscribe(topic: &str);

    #[wasm_bindgen(js_name = "nameResolve", catch)]
    pub async fn ipfs_name_resolve(cid: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "dagGet", catch)]
    pub async fn ipfs_dag_get(cid: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "dagGet", catch)]
    pub async fn ipfs_dag_get_path(cid: &str, path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "cat", catch)]
    pub async fn ipfs_cat(path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "getContenthash", catch)]
    pub async fn ens_get_content_hash(path: &str) -> Result<JsValue, JsValue>;
}
