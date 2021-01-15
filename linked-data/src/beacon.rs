use crate::{FakeCid, IPLDLink, RAW};

use serde::{Deserialize, Serialize};

use cid::Cid;
use multihash::Multihash;

/// Beacon pediodically send up to date video list cid crypto-signed
#[derive(Deserialize, Serialize)]
pub struct Beep {
    pub list: IPLDLink,
    pub signature: String,
}

/// List of video metadata plus update count
#[derive(Deserialize, Serialize)]
pub struct VideoList {
    pub counter: usize, // total number of video posted. Can ONLY go up. used to determine most recent update
    pub metadata: Vec<IPLDLink>, // oldest to newest
}

/// Video metadata
#[derive(Deserialize, Serialize, Clone)]
pub struct VideoMetadata {
    pub title: String,
    pub duration: f64,
    pub image: IPLDLink,
    pub video: IPLDLink,
}

//Hack

#[derive(Deserialize)]
pub struct TempVideoList {
    pub counter: usize,
    pub metadata: Vec<FakeCid>,
}

impl TempVideoList {
    pub fn into_video_list(self) -> VideoList {
        let mut metadata = Vec::with_capacity(self.metadata.len());

        for fake_cid in self.metadata.into_iter() {
            let multihash =
                Multihash::from_bytes(&fake_cid.hash.data).expect("Can't get multihash");

            let cid = Cid::new_v1(RAW, multihash);

            metadata.push(IPLDLink { link: cid });
        }

        VideoList {
            counter: self.counter,
            metadata,
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

impl TempVideoMetadata {
    pub fn into_metadata(self) -> VideoMetadata {
        let multihash = Multihash::from_bytes(&self.image.hash.data).expect("Can't get multihash");

        let cid = Cid::new_v1(RAW, multihash);

        let image = IPLDLink { link: cid };

        let multihash = Multihash::from_bytes(&self.video.hash.data).expect("Can't get multihash");

        let cid = Cid::new_v1(RAW, multihash);

        let video = IPLDLink { link: cid };

        VideoMetadata {
            title: self.title,
            duration: self.duration,
            image,
            video,
        }
    }
}
