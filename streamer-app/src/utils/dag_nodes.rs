use std::convert::TryFrom;
use std::io::Cursor;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::Serialize;

use cid::Cid;

/// Serialize then add dag node to IPFS. Return a CID.
pub async fn ipfs_dag_put_node_async<T>(ipfs: &IpfsClient, node: &T) -> Result<Cid, Error>
where
    T: ?Sized + Serialize,
{
    #[cfg(debug_assertions)]
    println!(
        "Serialize => {}",
        serde_json::to_string_pretty(node).unwrap()
    );

    let json_string = serde_json::to_string(node).expect("Serialization failed");

    let response = ipfs.dag_put(Cursor::new(json_string)).await?;

    let cid = Cid::try_from(response.cid.cid_string).expect("Invalid Cid");

    #[cfg(debug_assertions)]
    println!("IPFS: dag put => {}", &cid);

    Ok(cid)
}
