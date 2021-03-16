use std::collections::HashMap;

use crate::{FakeCid, IPLDLink, DAG_CBOR, RAW};

use serde::{Deserialize, Serialize};

use cid::Cid;
use multihash::Multihash;

/// Metadata for video thumbnails and playback.
#[derive(Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct VideoMetadata {
    pub title: String,
    pub duration: f64, // Must be less than actual video duratio, 0.1s less does it
    pub image: IPLDLink,
    pub video: IPLDLink, // TimecodeNode
}

/// Root CID.
#[derive(Serialize, Deserialize, Debug)]
pub struct TimecodeNode {
    /// ../time/..
    #[serde(rename = "time")]
    pub timecode: IPLDLink,
}

/// Links all hour nodes for multiple hours of video.
#[derive(Serialize, Deserialize, Debug)]
pub struct DayNode {
    /// ../time/hour/1/..
    #[serde(rename = "hour")]
    pub links_to_hours: Vec<IPLDLink>,
}

/// Links all minute nodes for 1 hour of video.
#[derive(Serialize, Deserialize, Debug)]
pub struct HourNode {
    /// ../time/hour/0/minute/15/..
    #[serde(rename = "minute")]
    pub links_to_minutes: Vec<IPLDLink>,
}

/// Links all variants nodes for 1 minute of video.
#[derive(Serialize, Deserialize, Debug)]
pub struct MinuteNode {
    /// ..time/hour/2/minute/36/second/30/..
    #[serde(rename = "second")]
    pub links_to_seconds: Vec<IPLDLink>,
}

/// Links video and chat nodes.
#[derive(Serialize, Deserialize, Debug)]
pub struct SecondNode {
    /// ../time/hour/3/minute/59/second/48/video/..
    #[serde(rename = "video")]
    pub link_to_video: IPLDLink,

    /// ../time/hour/4/minute/27/second/14/chat/0/..
    #[serde(rename = "chat")]
    pub links_to_chat: Vec<IPLDLink>,
}

/// Links all variants, allowing selection of video quality. Also link to the previous video node.
#[derive(Serialize, Deserialize, Debug)]
pub struct VideoNode {
    /// ../time/hour/0/minute/36/second/12/video/quality/1080p60/..
    #[serde(rename = "quality")]
    pub qualities: HashMap<String, IPLDLink>,

    /// ../time/hour/0/minute/36/second/12/video/setup/..
    #[serde(rename = "setup")]
    pub setup: Option<IPLDLink>,

    /// ../time/hour/0/minute/36/second/12/video/previous/..
    #[serde(rename = "previous")]
    pub previous: Option<IPLDLink>,
}

/// Codecs, qualities & initialization segments from lowest to highest quality.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetupNode {
    /// ../time/hour/0/minute/36/second/12/video/setup/quality
    #[serde(rename = "quality")]
    pub qualities: Vec<String>,

    /// ../time/hour/0/minute/36/second/12/video/setup/codec
    #[serde(rename = "codec")]
    pub codecs: Vec<String>,

    /// ../time/hour/0/minute/36/second/12/video/setup/initseg/0/..
    #[serde(rename = "initseg")]
    pub initialization_segments: Vec<IPLDLink>,

    /// ../time/hour/0/minute/36/second/12/video/setup/initseg/0/..
    pub bandwidths: Vec<usize>,
}

//Hack is needed to get from JsValue to Rust type via js http api

//TODO fix this hack
//Maybe work only with cbor as binary might be easier for Js <-> WASM interop

impl From<TempSetupNode> for SetupNode {
    fn from(temp: TempSetupNode) -> Self {
        let mut initialization_segments = Vec::with_capacity(temp.initialization_segments.len());

        for fake_cid in temp.initialization_segments.into_iter() {
            let multihash =
                Multihash::from_bytes(&fake_cid.hash.data).expect("Can't get multihash");

            let cid = Cid::new_v1(RAW, multihash);

            initialization_segments.push(IPLDLink { link: cid });
        }

        Self {
            codecs: temp.codecs,
            qualities: temp.qualities,
            initialization_segments,
            bandwidths: temp.bandwidths,
        }
    }
}

#[derive(Deserialize)]
pub struct TempSetupNode {
    #[serde(rename = "codec")]
    pub codecs: Vec<String>,

    #[serde(rename = "initseg")]
    pub initialization_segments: Vec<FakeCid>,

    #[serde(rename = "quality")]
    pub qualities: Vec<String>,

    pub bandwidths: Vec<usize>,
}

impl From<TempVideoMetadata> for VideoMetadata {
    fn from(temp: TempVideoMetadata) -> Self {
        let multihash = Multihash::from_bytes(&temp.image.hash.data).expect("Can't get multihash");

        let cid = Cid::new_v1(RAW, multihash);

        let image = IPLDLink { link: cid };

        let multihash = Multihash::from_bytes(&temp.video.hash.data).expect("Can't get multihash");

        let cid = Cid::new_v1(DAG_CBOR, multihash);

        let video = IPLDLink { link: cid };

        Self {
            title: temp.title,
            duration: temp.duration,
            image,
            video,
        }
    }
}

#[derive(Deserialize)]
pub struct TempVideoMetadata {
    pub title: String,
    pub duration: f64,
    pub image: FakeCid,
    pub video: FakeCid,
}
