#![allow(dead_code)]

use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use serde::{Deserialize, Serialize};

use web_sys::console;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub data: String,
}

#[wasm_bindgen(js_name = pubsubMessage)]
pub async fn pubsub_message(msg: JsValue) {
    /* let msg: Message = match msg.into_serde() {
        Ok(value) => value,
        Err(_) => {
            //TODO error
            return;
        }
    }; */

    console::log_1(&msg);
}
