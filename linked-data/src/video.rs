use std::collections::HashMap;

use crate::{FakeCid, IPLDLink, DAG_CBOR, RAW};

use serde::{Deserialize, Serialize};

use cid::Cid;
use multihash::Multihash;

/// Metadata for video thumbnails and playback.
/// Should not be pinned recursively.
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
    /// ../time/hour/0/minute/36/second/12/video/track/1080p60/..
    #[serde(rename = "track")]
    pub tracks: HashMap<String, IPLDLink>,

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
    /// ../time/hour/0/minute/36/second/12/video/setup/length
    #[serde(rename = "length")]
    pub segment_length: usize,

    /// ../time/hour/0/minute/36/second/12/video/setup/track/0/..
    #[serde(rename = "track")]
    pub tracks: Vec<Track>,
}

/// Codecs, qualities & initialization segments from lowest to highest quality.
#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    /// ../time/hour/0/minute/36/second/12/video/setup/track/2/quality
    pub quality: String,

    /// ../time/hour/0/minute/36/second/12/video/setup/track/3/codec
    pub codec: String,

    /// ../time/hour/0/minute/36/second/12/video/setup/track/2/initseg
    #[serde(rename = "initseg")]
    pub initialization_segment: IPLDLink,

    /// ../time/hour/0/minute/36/second/12/video/setup/track/3/bandwidth
    pub bandwidth: usize,
}
