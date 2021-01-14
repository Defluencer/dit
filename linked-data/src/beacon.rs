use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Beacon pediodically send up to date video list cid crypto-signed
#[derive(Deserialize, Serialize)]
pub struct Beep {
    pub list: IPLDLink,
    pub signature: String,
}

/// List of video metadata plus update count
#[derive(Deserialize, Serialize, Debug)]
pub struct VideoList {
    pub counter: usize, // total number of video posted. Can ONLY go up. used to determine most recent update
    pub metadata: Vec<IPLDLink>, // oldest to newest
}

/// Video metadata
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VideoMetadata {
    pub title: String,
    pub duration: f64,
    pub image: IPLDLink,
    pub video: IPLDLink,
}
