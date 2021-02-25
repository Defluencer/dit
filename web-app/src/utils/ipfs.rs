use ipfs_api::IpfsClient;

use std::convert::TryFrom;

use futures_util::TryStreamExt;

use yew::services::ConsoleService;
use yew::Callback;

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
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    if let Err(e) = source_buffer.append_buffer_with_u8_array(&mut buffer) {
        ConsoleService::warn(&format!("{:#?}", e));
        return;
    }
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

pub async fn ipfs_dag_get_path_async<T>(ipfs: IpfsClient, path: &str) -> Result<T, ()>
where
    T: for<'a> serde::Deserialize<'a>,
{
    let result = match ipfs
        .dag_get(&path)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return Err(());
        }
    };

    let node = serde_json::from_slice::<T>(&result).expect("Invalid Dag Node");

    Ok(node)
}

pub async fn ipfs_dag_get_callback<T>(ipfs: IpfsClient, cid: Cid, cb: Callback<(Cid, T)>)
where
    T: for<'a> serde::Deserialize<'a>,
{
    let result = match ipfs
        .dag_get(&cid.to_string())
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:#?}", e));
            return;
        }
    };

    let node = serde_json::from_slice::<T>(&result).expect("Invalid Dag Node");

    cb.emit((cid, node));
}
