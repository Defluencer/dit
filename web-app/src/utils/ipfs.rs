use crate::utils::bindings::{ipfs_cat, ipfs_dag_get, ipfs_dag_get_path, ipfs_name_resolve};

use std::convert::TryFrom;

use wasm_bindgen::JsCast;

use yew::services::ConsoleService;
use yew::Callback;

use js_sys::Uint8Array;
use web_sys::SourceBuffer;

use cid::Cid;

pub async fn cat_and_buffer(path: String, source_buffer: SourceBuffer) {
    let segment = match ipfs_cat(&path).await {
        Ok(vs) => vs,
        Err(e) => {
            ConsoleService::warn(&format!("{:#?}", e));
            return;
        }
    };

    let segment: &Uint8Array = segment.unchecked_ref();

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(segment) {
        ConsoleService::warn(&format!("{:#?}", e));
        return;
    }
}

pub async fn ipfs_resolve_and_get_callback<T>(ipns: String, cb: Callback<(Cid, T)>)
where
    T: for<'a> serde::Deserialize<'a>,
{
    let js_value = match ipfs_name_resolve(&ipns).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let path = match js_value.as_string() {
        Some(string) => string,
        None => return,
    };

    let cid = Cid::try_from(path).expect("Invalid Cid");

    let node = match ipfs_dag_get(&cid.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let node: T = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    cb.emit((cid, node));
}

pub async fn ipfs_dag_get_path_async<T>(cid: Cid, path: &str) -> Result<T, ()>
where
    T: for<'a> serde::Deserialize<'a>,
{
    let node = match ipfs_dag_get_path(&cid.to_string(), path).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return Err(());
        }
    };

    let node: T = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return Err(());
        }
    };

    Ok(node)
}

pub async fn ipfs_dag_get_callback<T>(cid: Cid, cb: Callback<(Cid, T)>)
where
    T: for<'a> serde::Deserialize<'a>,
{
    let node = match ipfs_dag_get(&cid.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let node: T = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    cb.emit((cid, node));
}
