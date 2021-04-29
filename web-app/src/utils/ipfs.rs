use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::Cursor;

use crate::utils::local_storage::{get_local_ipfs_addrs, get_local_storage, set_local_ipfs_addrs};

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;
use ipfs_api::TryFromUri;

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
pub struct IPFSService {
    client: IpfsClient,
}

impl IPFSService {
    pub fn new() -> Self {
        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let addrs = match get_local_ipfs_addrs(storage.as_ref()) {
            Some(addrs) => &addrs,
            None => {
                set_local_ipfs_addrs(DEFAULT_URI, storage.as_ref());

                DEFAULT_URI
            }
        };

        let uri = Uri::from_static(addrs);

        let client = IpfsClient::build_with_base_uri(uri);

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

    pub async fn resolve_and_dag_get<U, T>(
        &self,
        ipns: U,
        cb: Callback<(Cid, T)>,
    ) -> Result<(Cid, T), Error>
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
}
