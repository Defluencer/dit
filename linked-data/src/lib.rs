pub mod beacon;
pub mod chat;
pub mod config;
pub mod moderation;
pub mod video;

use crate::chat::UnsignedMessage;
use crate::moderation::Ban;

use std::convert::TryFrom;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use cid::Cid;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IPLDLink {
    #[serde(rename = "/")]
    #[serde(serialize_with = "serialize_cid")]
    #[serde(deserialize_with = "deserialize_cid")]
    pub link: Cid,
}

impl From<Cid> for IPLDLink {
    fn from(cid: Cid) -> Self {
        Self { link: cid }
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

    let cid = Cid::try_from(cid_str).expect("Deserialize string to CID failed");

    Ok(cid)
}

#[derive(Deserialize, Serialize)]
pub enum MessageType {
    Unsigned(UnsignedMessage),
    Ban(Ban),
}

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub msg_type: MessageType,

    /// Link to signed message.
    pub origin: IPLDLink,
}
