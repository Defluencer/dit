use std::borrow::Cow;
use std::convert::TryFrom;

use ipfs_api::IpfsClient;

use futures_util::TryStreamExt;

use futures::join;

use yew::services::ConsoleService;
use yew::Callback;

use cid::Cid;

//TODO pubsub

pub async fn init_cat(ipfs: IpfsClient, cid: Cid, cb: Callback<(Option<Vec<u8>>, Vec<u8>)>) {
    let data = match ipfs
        .cat(&cid.to_string())
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(data) => data,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    cb.emit((None, data));
}

pub async fn audio_video_cat(
    ipfs: IpfsClient,
    audio_path: String,
    video_path: String,
    cb: Callback<(Option<Vec<u8>>, Vec<u8>)>,
) {
    let (audio_res, video_res) = join!(
        ipfs.cat(&audio_path)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat(),
        ipfs.cat(&video_path)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
    );

    let audio_data = match audio_res {
        Ok(data) => data,
        Err(e) => {
            ConsoleService::warn(&format!("{:#?}", e));
            return;
        }
    };

    let video_data = match video_res {
        Ok(data) => data,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    cb.emit((Some(audio_data), video_data));
}

pub async fn ipfs_resolve_and_get_callback<T>(
    ipfs: IpfsClient,
    ipns: String,
    cb: Callback<(Cid, T)>,
) where
    T: for<'a> serde::Deserialize<'a>,
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

pub async fn ipfs_dag_get_callback<T>(ipfs: IpfsClient, cid: Cid, cb: Callback<(Cid, T)>)
where
    T: for<'a> serde::Deserialize<'a>,
{
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

pub async fn ipfs_dag_get_path_callback<U, T, K>(
    ipfs: IpfsClient,
    cid: Cid,
    path: U,
    cb: Callback<T>,
) where
    U: Into<Cow<'static, str>>,
    T: for<'a> serde::Deserialize<'a>,
{
    let path = format!("{}/{}", cid.to_string(), path.into());

    let node = match ipfs
        .dag_get(&path)
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

    cb.emit(node);
}
