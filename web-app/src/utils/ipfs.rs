use crate::utils::bindings::{ipfs_cat, ipfs_dag_get, ipfs_dag_get_path, ipfs_name_resolve};

use std::convert::TryFrom;
use std::path::PathBuf;

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

pub async fn ipfs_resolve_and_get_callback<T, U>(ipns: String, cb: Callback<(Cid, U)>)
where
    T: for<'a> serde::Deserialize<'a>,
    U: for<'a> From<T>,
{
    let js_value = match ipfs_name_resolve(&ipns).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let node = match ipfs_dag_get(&ipns).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let temp: T = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let node = temp.into();

    let path = match js_value.as_string() {
        Some(string) => string,
        None => return,
    };

    let path = PathBuf::try_from(path).expect("Invalid Path");
    let file_name = path.file_name().expect("Invalid File Name");
    let string = file_name.to_str().expect("Invalid Unicode");
    let cid = Cid::try_from(string).expect("Invalid Cid");

    cb.emit((cid, node));
}

pub async fn ipfs_dag_get_path_async<T, U>(cid: Cid, path: &str) -> Result<U, ()>
where
    T: for<'a> serde::Deserialize<'a>,
    U: for<'a> From<T>,
{
    let node = match ipfs_dag_get_path(&cid.to_string(), path).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return Err(());
        }
    };

    let temp: T = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return Err(());
        }
    };

    Ok(temp.into())
}

pub async fn ipfs_dag_get_callback<T, U>(cid: Cid, cb: Callback<(Cid, U)>)
where
    T: for<'a> serde::Deserialize<'a>,
    U: for<'a> From<T>,
{
    let node = match ipfs_dag_get(&cid.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let temp: T = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let node = temp.into();

    cb.emit((cid, node));
}
