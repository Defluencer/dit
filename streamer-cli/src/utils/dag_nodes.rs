use ipfs_api::response::KeyListResponse;
use ipfs_api::response::KeyPair;
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

/// Serialize the new node, pin it then publish it under this IPNS key.
pub async fn update_ipns<T>(ipfs: &IpfsClient, key: &str, content: &T) -> Result<(), Error>
where
    T: ?Sized + Serialize,
{
    let cid = ipfs_dag_put_node_async(ipfs, content).await?.to_string();

    ipfs.pin_add(&cid, false).await?;

    ipfs.name_publish(&cid, true, Some("4320h"), None, Some(key)) // 6 months
        .await?;

    Ok(())
}

/// Get node associated with IPNS key, unpin it then return it.
pub async fn get_from_ipns<T>(ipfs: &IpfsClient, key: &str) -> Result<T, Error>
where
    T: ?Sized + DeserializeOwned + Serialize,
{
    let mut res = ipfs.key_list().await?;

    let keypair = match search_keypairs(&key, &mut res) {
        Some(kp) => kp,
        None => return Err(Error::Uncategorized("Key Not Found".into())),
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", &keypair.name, &keypair.id);

    let res = ipfs.name_resolve(Some(&keypair.id), false, false).await?;

    let cid = Cid::try_from(res.path).expect("Invalid Cid");

    ipfs.pin_rm(&cid.to_string(), false).await?;

    let node = ipfs_dag_get_node_async(ipfs, &cid.to_string()).await?;

    Ok(node)
}

pub fn search_keypairs(name: &str, res: &mut KeyListResponse) -> Option<KeyPair> {
    for (i, keypair) in res.keys.iter_mut().enumerate() {
        if keypair.name == name {
            return Some(res.keys.remove(i));
        }
    }

    None
}
