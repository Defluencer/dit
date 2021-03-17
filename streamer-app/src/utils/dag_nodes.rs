use std::convert::TryFrom;
use std::io::Cursor;

use futures_util::TryStreamExt;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::de::DeserializeOwned;
use serde::Serialize;

use cid::Cid;

/// Serialize then add dag node to IPFS. Return a CID.
pub async fn ipfs_dag_put_node_async<T>(ipfs: &IpfsClient, node: &T) -> Result<Cid, Error>
where
    T: ?Sized + Serialize,
{
    #[cfg(debug_assertions)]
    println!(
        "Serde: Serialize => {}",
        serde_json::to_string_pretty(node).unwrap()
    );

    let json_string = serde_json::to_string(node).expect("Serialization failed");

    let response = ipfs.dag_put(Cursor::new(json_string)).await?;

    let cid = Cid::try_from(response.cid.cid_string).expect("Invalid Cid");

    #[cfg(debug_assertions)]
    println!("IPFS: dag put => {}", &cid);

    Ok(cid)
}

/// Deserialize dag node from IPFS path. Return dag node.
pub async fn ipfs_dag_get_node_async<T>(ipfs: &IpfsClient, path: &str) -> Result<T, Error>
where
    T: ?Sized + DeserializeOwned + Serialize,
{
    #[cfg(debug_assertions)]
    println!("IPFS: dag get => {}", path);

    let data = ipfs
        .dag_get(path)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await?;

    let node = serde_json::from_slice::<T>(&data).expect("Deserialization failed");

    #[cfg(debug_assertions)]
    println!(
        "Serde: Deserialize => {}",
        serde_json::to_string_pretty(&node).unwrap()
    );

    Ok(node)
}
