use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::Cursor;
use std::str::FromStr;

use crate::utils::local_storage::{get_local_ipfs_addrs, get_local_storage, set_local_ipfs_addrs};

use ipfs_api::response::Error;
use ipfs_api::response::PubsubSubResponse;
use ipfs_api::IpfsClient;
use ipfs_api::TryFromUri;

use futures::Stream;
use futures::StreamExt;
use futures_util::TryStreamExt;

use futures::join;

use serde::de::DeserializeOwned;
use serde::Serialize;

use yew::services::ConsoleService;
use yew::Callback;

use cid::Cid;

use http::Uri;

const DEFAULT_URI: &str = "http://localhost:5001/api/v0";

#[derive(Clone)]
pub struct IpfsService {
    client: IpfsClient,
}

impl IpfsService {
    pub fn new() -> Self {
        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let mut uri = None;

        if let Some(addrs) = get_local_ipfs_addrs(storage.as_ref()) {
            if let Ok(uri_from_str) = Uri::from_str(&addrs) {
                uri = Some(uri_from_str);
            }
        }

        if uri.is_none() {
            set_local_ipfs_addrs(DEFAULT_URI, storage.as_ref());

            uri = Some(Uri::from_static(DEFAULT_URI));
        }

        let client = IpfsClient::build_with_base_uri(uri.unwrap());

        Self { client }
    }

    pub async fn cid_cat(&self, cid: Cid) -> Result<Vec<u8>, Error> {
        self.client
            .cat(&cid.to_string())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
    }

    pub async fn double_path_cat<U>(
        &self,
        audio_path: U,
        video_path: U,
    ) -> Result<(Vec<u8>, Vec<u8>), Error>
    where
        U: Into<Cow<'static, str>>,
    {
        let (audio_res, video_res) = join!(
            self.client
                .cat(&audio_path.into())
                .map_ok(|chunk| chunk.to_vec())
                .try_concat(),
            self.client
                .cat(&video_path.into())
                .map_ok(|chunk| chunk.to_vec())
                .try_concat()
        );

        let audio_data = audio_res?;
        let video_data = video_res?;

        Ok((audio_data, video_data))
    }

    pub async fn resolve_and_dag_get<U, T>(&self, ipns: U) -> Result<(Cid, T), Error>
    where
        U: Into<Cow<'static, str>>,
        T: ?Sized + DeserializeOwned,
    {
        let res = self
            .client
            .name_resolve(Some(&ipns.into()), true, false)
            .await?;

        let cid = Cid::try_from(res.path).expect("Invalid Cid");

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("IPFS: name resolve => {}", cid.to_string()));

        let res = self
            .client
            .dag_get(&cid.to_string())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;

        let node = serde_json::from_slice::<T>(&res).expect("Invalid Dag Node");

        Ok((cid, node))
    }

    /// Serialize then add dag node to IPFS. Return a CID.
    pub async fn dag_put<T>(&self, node: &T) -> Result<Cid, Error>
    where
        T: ?Sized + Serialize,
    {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Serde: Serialize => {}",
            serde_json::to_string_pretty(node).unwrap()
        ));

        let json_string = serde_json::to_string(node).expect("Serialization failed");

        let response = self.client.dag_put(Cursor::new(json_string)).await?;

        let cid = Cid::try_from(response.cid.cid_string).expect("Invalid Cid");

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("IPFS: dag put => {}", &cid));

        Ok(cid)
    }

    /// Deserialize dag node from IPFS path. Return dag node.
    pub async fn dag_get<U, T>(&self, cid: Cid, path: Option<U>) -> Result<T, Error>
    where
        U: Into<Cow<'static, str>>,
        T: ?Sized + DeserializeOwned,
    {
        let mut origin = cid.to_string();

        if let Some(path) = path {
            origin.push_str(&path.into());
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("IPFS: dag get => {}", origin));

        let result = self
            .client
            .dag_get(&origin)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await?;

        let node = serde_json::from_slice::<T>(&result).expect("Invalid Dag Node");

        Ok(node)
    }

    pub async fn pubsub_sub<U>(&self, topic: U, cb: Callback<Result<PubsubSubResponse, Error>>)
    where
        U: Into<Cow<'static, str>>,
    {
        let mut stream = self.client.pubsub_sub(&topic.into(), true);

        while let Some(result) = stream.next().await {
            cb.emit(result);
        }
    }
}
