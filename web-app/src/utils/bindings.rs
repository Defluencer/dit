use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen(module = "/libs.js")]
extern "C" {
    #[wasm_bindgen(js_name = "getContenthash", catch)]
    pub fn ens_get_content_hash(path: &str) -> Result<JsValue, JsValue>;
}
