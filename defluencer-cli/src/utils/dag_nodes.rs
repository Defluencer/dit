use ipfs_api::response::KeyPair;
use std::convert::TryFrom;
use std::io::Cursor;

use futures_util::TryStreamExt;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::de::DeserializeOwned;
use serde::Serialize;

use cid::Cid;

/// Serialize then add dag node to IPFS and return CID.
pub async fn ipfs_dag_put_node_async<T>(ipfs: &IpfsClient, node: &T) -> Result<Cid, Error>
where
    T: ?Sized + Serialize,
{
    #[cfg(debug_assertions)]
    println!(
        "Serde: Serialize => {}",
        serde_json::to_string_pretty(node).unwrap()
    );

    let json_string = serde_json::to_string(node).expect("Cannot Serialize");

    let response = ipfs.dag_put(Cursor::new(json_string)).await?;

    let cid = Cid::try_from(response.cid.cid_string).expect("Invalid Cid");

    #[cfg(debug_assertions)]
    println!("IPFS: dag put => {}", &cid);

    Ok(cid)
}

/// Deserialize dag node from IPFS path and return.
pub async fn ipfs_dag_get_node_async<T>(ipfs: &IpfsClient, path: &str) -> Result<T, Error>
where
    T: ?Sized + DeserializeOwned,
{
    #[cfg(debug_assertions)]
    println!("IPFS: dag get => {}", path);

    let data = ipfs
        .dag_get(path)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await?;

    #[cfg(debug_assertions)]
    println!(
        "Serde: Deserialize => {}",
        std::str::from_utf8(&data).expect("Not UTF-8 Data")
    );

    let node = serde_json::from_slice::<T>(&data).expect("Cannot Deserialize");

    Ok(node)
}

/// Serialize the new node, direct pin then publish under this IPNS key.
pub async fn update_ipns<T>(ipfs: &IpfsClient, key: &str, content: &T) -> Result<(), Error>
where
    T: ?Sized + Serialize,
{
    let cid = ipfs_dag_put_node_async(ipfs, content).await?;

    let cid = cid.to_string();

    if let Err(e) = ipfs.pin_add(&cid, false).await {
        eprintln!("❗ IPFS could not pin {}. Error: {}", cid, e);
    }

    if cfg!(debug_assertions) {
        ipfs.name_publish(&cid, true, None, None, Some(key)).await?;
    } else {
        ipfs.name_publish(&cid, true, Some("4320h"), None, Some(key)) // 6 months
            .await?;
    }

    Ok(())
}

/// Get node associated with IPNS key, direct unpin then return.
pub async fn get_from_ipns<T>(ipfs: &IpfsClient, key: &str) -> Result<(Cid, T), Error>
where
    T: ?Sized + DeserializeOwned,
{
    let res = ipfs.key_list().await?;

    let keypair = match search_keypairs(key, &res) {
        Some(keypair) => keypair,
        None => return Err(Error::Uncategorized("Key Not Found".into())),
    };

    #[cfg(debug_assertions)]
    println!("IPNS: key => {} {}", &keypair.name, &keypair.id);

    let res = ipfs.name_resolve(Some(&keypair.id), false, false).await?;
    let cid = Cid::try_from(res.path).expect("Invalid Cid");

    let node = ipfs_dag_get_node_async(ipfs, &cid.to_string()).await?;

    Ok((cid, node))
}

pub fn search_keypairs<'a>(
    name: &str,
    res: &'a ipfs_api::response::KeyPairList,
) -> Option<&'a KeyPair> {
    for keypair in res.keys.iter() {
        if keypair.name == name {
            return Some(keypair);
        }
    }

    None
}
