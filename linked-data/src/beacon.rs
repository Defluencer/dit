use crate::IPLDLink;

use serde::{Deserialize, Serialize};

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
