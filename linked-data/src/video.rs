use crate::{FakeCid, IPLDLink, DAG_CBOR, RAW};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use cid::Cid;
use multihash::Multihash;

/// Metadata for video thumbnails and playback.
#[derive(Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct VideoMetadata {
    pub title: String,
    pub duration: f64, // Must be less than actual video duration, 0.1s less does it
    pub image: IPLDLink, // Raw node of image
    pub video: IPLDLink, // TimecodeNode
                       //creator identity whatever that may be
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

/// Links all stream variants, allowing selection of video quality. Also link to the previous video node.
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

/// Contains initialization data for video stream.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetupNode {
    /// Tracks sorted from lowest to highest bitrate.
    #[serde(rename = "track")]
    pub tracks: Vec<Track>, // ../time/hour/0/minute/36/second/12/video/setup/track/0/..
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub name: String,  // ../time/hour/0/minute/36/second/12/video/setup/track/2/name
    pub codec: String, // ../time/hour/0/minute/36/second/12/video/setup/track/3/codec

    #[serde(rename = "initseg")]
    pub initialization_segment: IPLDLink, // ../time/hour/0/minute/36/second/12/video/setup/track/1/initseg

    pub bandwidth: usize, // ../time/hour/0/minute/36/second/12/video/setup/track/4/bandwidth
}

//Hack is needed to get from JsValue to Rust type via js http api

//TODO fix this hack
//Maybe work only with cbor as binary might be easier for Js <-> WASM interop

#[derive(Deserialize)]
pub struct TempTrack {
    pub name: String,
    pub codec: String,

    #[serde(rename = "initseg")]
    pub initialization_segment: FakeCid,

    pub bandwidth: usize,
}

#[derive(Deserialize)]
pub struct TempSetupNode {
    #[serde(rename = "track")]
    pub tracks: Vec<TempTrack>,
}

impl From<TempSetupNode> for SetupNode {
    fn from(temp: TempSetupNode) -> Self {
        let mut tracks = Vec::with_capacity(temp.tracks.len());

        for track in temp.tracks.into_iter() {
            let multihash = Multihash::from_bytes(&track.initialization_segment.hash.data)
                .expect("Can't get multihash");

            let link = IPLDLink {
                link: Cid::new_v1(RAW, multihash),
            };

            tracks.push(Track {
                name: track.name,
                codec: track.codec,
                initialization_segment: link,
                bandwidth: track.bandwidth,
            });
        }

        Self { tracks }
    }
}

#[derive(Deserialize)]
pub struct TempVideoMetadata {
    pub title: String,
    pub duration: f64,
    pub image: FakeCid,
    pub video: FakeCid,
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
pub struct TempVideoNode {
    #[serde(rename = "track")]
    pub tracks: HashMap<String, FakeCid>,

    #[serde(rename = "setup")]
    pub setup: Option<FakeCid>,

    #[serde(rename = "previous")]
    pub previous: Option<FakeCid>,
}

impl From<TempVideoNode> for VideoNode {
    fn from(temp: TempVideoNode) -> Self {
        let mut tracks = HashMap::with_capacity(temp.tracks.len());

        for (name, fcid) in temp.tracks.into_iter() {
            let multihash = Multihash::from_bytes(&fcid.hash.data).expect("Can't get multihash");

            let cid = Cid::new_v1(DAG_CBOR, multihash);

            tracks.insert(name, IPLDLink { link: cid });
        }

        let setup = match temp.setup.as_ref() {
            Some(data) => {
                let multihash =
                    Multihash::from_bytes(&data.hash.data).expect("Can't get multihash");

                let cid = Cid::new_v1(DAG_CBOR, multihash);

                Some(IPLDLink { link: cid })
            }
            None => None,
        };

        let previous = match temp.previous.as_ref() {
            Some(data) => {
                let multihash =
                    Multihash::from_bytes(&data.hash.data).expect("Can't get multihash");

                let cid = Cid::new_v1(DAG_CBOR, multihash);

                Some(IPLDLink { link: cid })
            }
            None => None,
        };

        Self {
            tracks,
            setup,
            previous,
        }
    }
}
