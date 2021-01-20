pub mod beacon;
pub mod chat;
pub mod config;
pub mod stream;
pub mod video;

use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use cid::Cid;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct IPLDLink {
    #[serde(rename = "/")]
    #[serde(serialize_with = "serialize_cid")]
    #[serde(deserialize_with = "deserialize_cid")]
    pub link: Cid,
}

impl Default for IPLDLink {
    fn default() -> Self {
        Self {
            link: Cid::default(),
        }
    }
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

//TODO fix this mess...

//Hack to get around js api
//Have to deserialize into rust cid from js object representing cid

pub const RAW: u64 = 0x55;
pub const DAG_CBOR: u64 = 0x71;

#[derive(Deserialize)]
pub struct FakeCid {
    pub codec: String,
    pub version: u8,
    pub hash: Hash,
}

#[derive(Deserialize)]
pub struct Hash {
    #[serde(rename = "type")]
    pub hash_type: String,
    pub data: Vec<u8>,
}
