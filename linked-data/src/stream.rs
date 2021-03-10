use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Stream Root CID.
#[derive(Serialize, Deserialize, Debug)]
pub struct StreamNode {
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
