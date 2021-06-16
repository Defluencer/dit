use crate::PeerId;

use serde::{Deserialize, Serialize};

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
