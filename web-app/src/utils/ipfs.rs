use crate::utils::bindings::{ipfs_cat, ipfs_dag_get};

use wasm_bindgen::JsCast;

use yew::services::ConsoleService;
use yew::Callback;

use js_sys::Uint8Array;
use web_sys::SourceBuffer;

use linked_data::beacon::{TempVideoList, TempVideoMetadata, VideoList, VideoMetadata};

use cid::Cid;

pub async fn cat_and_buffer(path: String, source_buffer: SourceBuffer) {
    let segment = match ipfs_cat(&path).await {
        Ok(vs) => vs,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let segment: &Uint8Array = segment.unchecked_ref();

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(segment) {
        ConsoleService::warn(&format!("{:?}", e));
        return;
    }
}

/* pub async fn ipfs_dag_get_node_async<T>(cid: Cid, cb: Callback<(Cid, T)>)
where
    T: for<'a> serde::Deserialize<'a>,
{
    let node = match ipfs_dag_get(&cid.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let node: T = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    cb.emit((cid, node));
} */

pub async fn ipfs_dag_get_list(cid: Cid, cb: Callback<(Cid, VideoList)>) {
    let node = match ipfs_dag_get(&cid.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let temp: TempVideoList = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let node = temp.into_video_list();

    cb.emit((cid, node));
}

pub async fn ipfs_dag_get_metadata(cid: Cid, cb: Callback<(Cid, VideoMetadata)>) {
    let node = match ipfs_dag_get(&cid.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let temp: TempVideoMetadata = match node.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let node = temp.into_metadata();

    cb.emit((cid, node));
}
