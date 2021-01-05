use std::convert::TryFrom;
use std::io::Cursor;
use std::str::FromStr;

use ipfs_api::response::Error;
use ipfs_api::IpfsClient;

use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};

use cid::Cid;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct IPLDLink {
    #[serde(rename = "/")]
    #[serde(serialize_with = "serialize_cid")]
    #[serde(deserialize_with = "deserialize_cid")]
    pub link: Cid,
}

fn serialize_cid<S>(cid: &Cid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&cid.to_string())
}

fn deserialize_cid<'de, D>(deserializer: D) -> Result<Cid, D::Error>
where
    D: Deserializer<'de>,
{
    let cid_str: &str = Deserialize::deserialize(deserializer)?;

    let cid = Cid::from_str(cid_str).expect("Deserialize string to CID failed");

    Ok(cid)
}

impl Default for IPLDLink {
    fn default() -> Self {
        Self {
            link: Cid::default(),
        }
    }
}

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
    println!("Dag Put => {}", &cid);

    Ok(cid)
}
