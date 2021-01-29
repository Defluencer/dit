use crate::{FakeCid, IPLDLink, DAG_CBOR, RAW};

use serde::{Deserialize, Serialize};

use cid::Cid;
use multihash::Multihash;

/// Beacon pediodically send up to date video list cid crypto-signed
/* #[derive(Deserialize, Serialize)]
pub struct Beep {
    pub list: IPLDLink,
    pub signature: String,
} */

/// List of video metadata plus update count
#[derive(Deserialize, Serialize)]
pub struct VideoList {
    pub counter: usize, // total number of video posted. Can ONLY go up. used to determine most recent update
    pub metadata: Vec<IPLDLink>, // oldest to newest
}

/// Video metadata
#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct VideoMetadata {
    pub title: String,
    pub duration: f64,
    pub image: IPLDLink,
    pub video: IPLDLink,
}

//Hack is needed to get from JsValue to Rust type via js http api

impl From<TempVideoList> for VideoList {
    fn from(temp: TempVideoList) -> Self {
        let mut metadata = Vec::with_capacity(temp.metadata.len());

        for fake_cid in temp.metadata.into_iter() {
            let multihash =
                Multihash::from_bytes(&fake_cid.hash.data).expect("Can't get multihash");

            let cid = Cid::new_v1(DAG_CBOR, multihash);

            metadata.push(IPLDLink { link: cid });
        }

        Self {
            counter: temp.counter,
            metadata,
        }
    }
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
pub struct TempVideoList {
    pub counter: usize,
    pub metadata: Vec<FakeCid>,
}

#[derive(Deserialize)]
pub struct TempVideoMetadata {
    pub title: String,
    pub duration: f64,
    pub image: FakeCid,
    pub video: FakeCid,
}
