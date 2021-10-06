use crate::{IPLDLink, PeerId};

use crate::moderation::{Ban, Moderator};
use serde::{Deserialize, Serialize};

//TODO Think about archiving the messages and update accordingly.

/// Unsigned chat message.
#[derive(Serialize, Deserialize, Debug)]
pub struct UnsignedMessage {
    pub message: String,
}

/// Chat identifiers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatId {
    pub name: String,

    pub peer: PeerId,
}

#[derive(Deserialize, Serialize)]
pub enum MessageType {
    Unsigned(UnsignedMessage),
    Ban(Ban),
    Mod(Moderator),
}

/// GossipSub Live Chat Message.
#[derive(Deserialize, Serialize)]
pub struct Message {
    pub msg_type: MessageType,

    /// Link to signed message.
    pub origin: IPLDLink,
}
