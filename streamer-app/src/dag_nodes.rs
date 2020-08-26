use serde::Serializer;
use std::collections::HashMap;

use serde::Serialize;

use cid::Cid;
use multibase::Base;

#[derive(Serialize, Debug, Clone)]
pub struct IPLDLink {
    #[serde(serialize_with = "serialize_cid")]
    #[serde(rename = "/")]
    pub link: Cid,
}

fn serialize_cid<S>(cid: &Cid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string = cid
        .to_string_of_base(Base::Base32Lower)
        .expect("serialize_cid failed");

    serializer.serialize_str(&string)
}

/// Stream Root CID.
#[derive(Serialize, Debug)]
pub struct StreamNode {
    #[serde(rename = "time")]
    pub timecode: IPLDLink, // ../<StreamHash>/time/..
}

/// Links all hour nodes for multiple hours of video.
#[derive(Serialize, Debug)]
pub struct HoursNode {
    #[serde(rename = "hour")]
    pub links_to_minutes: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/..
}

/// Links all minute nodes for 1 hour of video.
#[derive(Serialize, Debug)]
pub struct MinutesNode {
    #[serde(rename = "minute")]
    pub links_to_seconds: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/..
}

/// Links all variants nodes for 1 minute of video.
#[derive(Serialize, Debug)]
pub struct SecondsNode {
    #[serde(rename = "second")]
    pub links_to_video: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/..
}

/// Link all stream variants.
/// Allow viewer to select video quality.
#[derive(Serialize, Debug)]
pub struct VariantsNode {
    #[serde(rename = "quality")]
    pub variants: HashMap<String, IPLDLink>, // ../<StreamHash>/time/hour/0/minute/36/second/12/quality/1080p60/..
}

/// Link the current stream variants dag node and the previous live dag node.
/// Allow rewind/buffer previous video segments.
#[derive(Serialize, Debug)]
pub struct LiveNode {
    pub current: IPLDLink,
    pub previous: Option<IPLDLink>,
}
