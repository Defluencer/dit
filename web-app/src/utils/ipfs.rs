use std::convert::TryFrom;
use std::borrow::Cow;
use std::convert::TryFrom;

use ipfs_api::IpfsClient;

use futures_util::TryStreamExt;

use futures::join;

use wasm_bindgen::JsCast;

use yew::services::ConsoleService;
use yew::Callback;

use js_sys::Uint8Array;

use web_sys::SourceBuffer;

use cid::Cid;

pub async fn cat_and_buffer(ipfs: IpfsClient, path: String, source_buffer: SourceBuffer) {
    let mut buffer = match ipfs
        .cat(&path)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(vec) => vec,
}

pub async fn init_cat(cid: Cid, cb: Callback<(Option<Uint8Array>, Uint8Array)>) {
    let res = ipfs_cat(&cid.to_string()).await;

    let js_value = match res {
        Ok(js) => js,
        Err(e) => {
            ConsoleService::warn(&format!("{:#?}", e));
            return;
        }
    };

    let video_seg: Uint8Array = js_value.unchecked_into();

    cb.emit((None, video_seg));
}

pub async fn audio_video_cat(
    audio_path: String,
    video_path: String,
    cb: Callback<(Option<Uint8Array>, Uint8Array)>,
) {
    let (audio_res, video_res) = join!(ipfs_cat(&audio_path), ipfs_cat(&video_path));

    let js_value = match audio_res {
        Ok(js) => js,
        Err(e) => {
            ConsoleService::warn(&format!("{:#?}", e));
            return;
        }
    };

    let audio_seg: Uint8Array = js_value.unchecked_into();

    let js_value = match video_res {
        Ok(js) => js,

        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

pub async fn ipfs_resolve_and_get_callback<T, K>(ipns: String, cb: Callback<(Cid, K)>)
where
    T: for<'a> serde::Deserialize<'a> + Into<K>,
    K: for<'a> serde::Deserialize<'a>,

{
    let res = match ipfs.name_resolve(Some(&ipns), true, false).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let cid = Cid::try_from(res.path).expect("Invalid Cid");


    let node = match ipfs
        .dag_get(&cid.to_string())
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(result) => serde_json::from_slice::<T>(&result).expect("Invalid Dag Node"),
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    cb.emit((cid, node));
}


pub async fn ipfs_dag_get_callback<T, K>(cid: Cid, cb: Callback<(Cid, K)>)
where
    T: for<'a> serde::Deserialize<'a> + Into<K>,
    K: for<'a> serde::Deserialize<'a>,
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

    cb.emit((cid, node.into()));
}

pub async fn ipfs_dag_get_path_callback<U, T, K>(cid: Cid, path: U, cb: Callback<K>)
where
    U: Into<Cow<'static, str>>,
    T: for<'a> serde::Deserialize<'a> + Into<K>,
    K: for<'a> serde::Deserialize<'a>,
{
    let node = match ipfs_dag_get_path(&cid.to_string(), &path.into()).await {

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

    cb.emit(node.into());
}
