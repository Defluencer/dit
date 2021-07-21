use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Topics {
    pub live_video: String,
    pub live_chat: String,
    pub comments: String,
}

/// Mostly static links to content.
/// Direct pin.
#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Clone)]
pub struct Beacon {
    /// Broadcaster GossipSub Topics.
    pub topics: Topics,

    /// Broadcaster GossipSub Peer ID.
    pub peer_id: String, // Base58btc encoded string.

    /// Link to all banned addresses.
    pub bans: String, //IPNS path -> "/ipns/<hash>"

    /// Link to all mods addresses.
    pub mods: String, //IPNS path -> "/ipns/<hash>"

    /// Link to all content metadata.
    pub content_feed: String, //IPNS path -> "/ipns/<hash>"

    // Link to all archived comments.
    pub comments: String, //IPNS path -> "/ipns/<hash>"
}
