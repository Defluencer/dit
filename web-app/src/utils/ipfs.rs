use crate::utils::bindings::{ipfs_cat, wait_until};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use yew::services::ConsoleService;

use js_sys::Uint8Array;
use web_sys::SourceBuffer;

pub async fn cat_and_buffer(path: String, source_buffer: SourceBuffer) {
    let segment = match ipfs_cat(&path).await {
        Ok(vs) => vs,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let segment: &Uint8Array = segment.unchecked_ref();

    //wait_for_buffer(source_buffer.clone()).await;

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(segment) {
        ConsoleService::warn(&format!("{:?}", e));
        return;
    }
}

async fn _wait_for_buffer(source_buffer: SourceBuffer) {
    let closure = move || !source_buffer.updating();

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn() -> bool>);

    wait_until(callback.into_js_value().unchecked_ref()).await
}
