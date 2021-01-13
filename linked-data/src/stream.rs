use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Stream Root CID.
#[derive(Serialize, Deserialize, Debug)]
pub struct StreamNode {
    #[serde(rename = "time")]
    pub timecode: IPLDLink, // ../<StreamHash>/time/..
}

/// Links all hour nodes for multiple hours of video.
#[derive(Serialize, Deserialize, Debug)]
pub struct DayNode {
    #[serde(rename = "hour")]
    pub links_to_hours: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/..
}

/// Links all minute nodes for 1 hour of video.
#[derive(Serialize, Deserialize, Debug)]
pub struct HourNode {
    #[serde(rename = "minute")]
    pub links_to_minutes: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/..
}

/// Links all variants nodes for 1 minute of video.
#[derive(Serialize, Deserialize, Debug)]
pub struct MinuteNode {
    #[serde(rename = "second")]
    pub links_to_seconds: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/..
}

/// Links video and chat nodes.
#[derive(Serialize, Deserialize, Debug)]
pub struct SecondNode {
    #[serde(rename = "video")]
    pub link_to_video: IPLDLink, // ../<StreamHash>/time/hour/1/minute/15/second/30/video/..

    #[serde(rename = "chat")]
    pub links_to_chat: Vec<IPLDLink>, // ../<StreamHash>/time/hour/1/minute/15/second/30/chat/0/..
}
