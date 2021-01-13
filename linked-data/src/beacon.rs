use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Beacon pediodically send up to date video list cid crypto-signed
#[derive(Deserialize, Serialize)]
pub struct Beep {
    pub list: IPLDLink,
    pub signature: String,
}

/// List of video metadata plus update count
#[derive(Deserialize, Serialize)]
pub struct VideoList {
    pub counter: usize,               // used to determine most recent
    pub metadata: Vec<VideoMetaData>, // newest to oldest
}

/// Video metadata
#[derive(Deserialize, Serialize, Clone)]
pub struct VideoMetaData {
    pub title: String,
    pub duration: f64,
    //thumbnail image cid, timestamp, geo-loc, etc...
    pub video: IPLDLink,
}
