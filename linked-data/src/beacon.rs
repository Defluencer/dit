use crate::{FakeCid, IPLDLink, DAG_CBOR};

use serde::{Deserialize, Serialize};

use cid::Cid;
use multihash::Multihash;

/// Mostly static list of links to content.
#[derive(Deserialize, Serialize)]
pub struct Beacon {
    /// GossipSub topics for live streaming & chat.
    pub topics: Topics,

    /// Base58btc encoded string.
    pub peer_id: String,

    // IPNS paths -> "/ipns/<hash>"
    pub video_list: String, //resolve to VideoList
                            //pub chat_mods: Option<String>,  //resolve to Moderators
                            //pub chat_block: Option<String>, //resolve to Blacklist
                            //pub chat_allow: Option<String>, //resolve to Whitelist
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Topics {
    pub live_video: String,
    pub live_chat: String,
}

/// List of all video metadata links.
#[derive(Deserialize, Serialize, Default)]
pub struct VideoList {
    /// Oldest to newest videos metadata.
    pub metadata: Vec<IPLDLink>,
}

//Hack is needed to get from JsValue to Rust type via js http api

//TODO fix this hack
//Maybe work only with cbor as binary might be easier for Js <-> WASM interop

impl From<TempVideoList> for VideoList {
    fn from(temp: TempVideoList) -> Self {
        let mut metadata = Vec::with_capacity(temp.metadata.len());

        for fake_cid in temp.metadata.into_iter() {
            let multihash =
                Multihash::from_bytes(&fake_cid.hash.data).expect("Can't get multihash");

            let cid = Cid::new_v1(DAG_CBOR, multihash);

            metadata.push(IPLDLink { link: cid });
        }

        Self { metadata }
    }
}

#[derive(Deserialize)]
pub struct TempVideoList {
    pub metadata: Vec<FakeCid>,
}
