use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Topics {
    pub live_video: String,
    pub live_chat: String,
}

/// Mostly static links to content.
#[derive(Deserialize, Serialize)]
pub struct Beacon {
    /// Broadcaster GossipSub Topics.
    pub topics: Topics,

    /// Broadcaster GossipSub Peer ID.
    pub peer_id: String, // Base58btc encoded string.

    /// Link to all video metadata.
    pub videos: String, //IPNS path -> "/ipns/<hash>"

    /// Link to all banned addresses.
    pub bans: String, //IPNS path -> "/ipns/<hash>"

    /// Link to all mods addresses.
    pub mods: String, //IPNS path -> "/ipns/<hash>"
}
