use crate::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Live {
    /// PubSub topic for the live streaming.
    pub video_topic: String,

    /// PubSub topic form the live chat.
    pub chat_topic: String,

    /// IPFS Peer ID. Base58btc.
    pub peer_id: PeerId,
}
