use std::collections::HashMap;
use std::str::FromStr;

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

/// Stream Root CID.
#[derive(Serialize, Debug)]
pub struct StreamNode {
    #[serde(rename = "time")]
    pub timecode: IPLDLink, // ../<StreamHash>/time/..
}

/// Links all hour nodes for multiple hours of video.
#[derive(Serialize, Debug)]
pub struct DayNode {
    #[serde(rename = "hour")]
    pub links_to_hours: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/..
}

/// Links all minute nodes for 1 hour of video.
#[derive(Serialize, Debug)]
pub struct HourNode {
    #[serde(rename = "minute")]
    pub links_to_minutes: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/..
}

/// Links all variants nodes for 1 minute of video.
#[derive(Serialize, Debug)]
pub struct MinuteNode {
    #[serde(rename = "second")]
    pub links_to_seconds: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/..
}

/// Links video and chat nodes.
#[derive(Serialize, Debug)]
pub struct SecondNode {
    #[serde(rename = "video")]
    pub link_to_video: IPLDLink, // ../<StreamHash>/time/hour/1/minute/15/second/30/video/..

    #[serde(rename = "chat")]
    pub links_to_chat: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/chat/0/..
}

/* Below are nodes created during the live stream */

/// Link all stream variants.
/// Allow viewer to select video quality.
#[derive(Serialize, Debug)]
pub struct VariantsNode {
    #[serde(rename = "quality")]
    pub variants: HashMap<String, IPLDLink>, // ../<StreamHash>/time/hour/0/minute/36/second/12/video/quality/1080p60/..
}

/// Link the current stream variants dag node and the previous live dag node.
/// Allow rewind/buffer previous video segments.
#[derive(Serialize, Debug)]
pub struct LiveNode {
    pub current: IPLDLink,
    pub previous: Option<IPLDLink>,
}

/// Chat message optionaly signed with some form of private key
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub public_key: Option<String>,

    pub signature: Option<String>,

    pub data: ChatContent,
}

/// Containts; user name, message and a link to VariantsNode as timestamp
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatContent {
    pub name: String,

    pub message: String,

    pub timestamp: IPLDLink,
}

/* Below are nodes used for chat moderation */

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct ChatIdentity {
    #[serde(rename = "PeerId")]
    pub peer_id: String, //TODO switch to CID when go-Ipfs 0.7 drops

    #[serde(rename = "PublicKey")]
    pub public_key: String, //TODO find crate with ETH address and signature types.
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ban {
    pub ban: ChatIdentity,
}
