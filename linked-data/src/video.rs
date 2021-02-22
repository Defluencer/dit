use std::collections::HashMap;

use crate::{FakeCid, IPLDLink, RAW};

use serde::{Deserialize, Serialize};

use cid::Cid;
use multihash::Multihash;

/// Links all variants, allowing selection of video quality. Also link to the previous video node.
#[derive(Serialize, Deserialize, Debug)]
pub struct VideoNode {
    // <StreamHash>/time/hour/0/minute/36/second/12/video/quality/1080p60/..
    #[serde(rename = "quality")]
    pub qualities: HashMap<String, IPLDLink>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/..
    #[serde(rename = "setup")]
    pub setup: Option<IPLDLink>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/previous/..
    #[serde(rename = "previous")]
    pub previous: Option<IPLDLink>,
}

/// Codecs, qualities & initialization segments from lowest to highest quality.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetupNode {
    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/quality
    #[serde(rename = "quality")]
    pub qualities: Vec<String>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/codec
    #[serde(rename = "codec")]
    pub codecs: Vec<String>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/initseg/0/..
    #[serde(rename = "initseg")]
    pub initialization_segments: Vec<IPLDLink>,

    // <StreamHash>/time/hour/0/minute/36/second/12/video/setup/initseg/0/..
    pub bandwidths: Vec<usize>,
}

//Hack is needed to get from JsValue to Rust type via js http api

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
