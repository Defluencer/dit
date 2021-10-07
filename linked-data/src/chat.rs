use crate::{IPLDLink, PeerId};

use crate::moderation::{Ban, Moderator};
use serde::{Deserialize, Serialize};

/// GossipSub Live Chat Message.
#[derive(Deserialize, Serialize)]
pub struct Message {
    pub msg: MessageType,

    /// Link to signed message.
    pub sig: IPLDLink,
}

#[derive(Deserialize, Serialize)]
pub enum MessageType {
    Chat(String),
    Ban(Ban),
    Mod(Moderator),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatId {
    pub name: String,

    pub peer_id: PeerId,
}
