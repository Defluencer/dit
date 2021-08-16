use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Topics {
    pub live_video: String,
    pub live_chat: String,
}

/// Mostly static links to content.
/// Direct pin.
#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Clone)]
pub struct Beacon {
    /// GossipSub Topics.
    pub topics: Topics,

    /// IPFS Peer ID. Base58btc.
    pub peer_id: String,

    /// Link to list of content metadata.
    pub content_feed: String, //IPNS path -> "/ipns/<hash>"

    /// Link to list of comments.
    pub comments: Option<String>, //IPNS path -> "/ipns/<hash>"

    /// Link to list of your friend's beacons.
    pub friends: Option<String>, //IPNS path -> "/ipns/<hash>"

    /// Link to all banned addresses.
    pub bans: Option<String>, //IPNS path -> "/ipns/<hash>"

    /// Link to all moderator addresses.
    pub mods: Option<String>, //IPNS path -> "/ipns/<hash>"
}
