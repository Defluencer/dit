use crate::IPLDLink;

use serde::{Deserialize, Serialize};

use crate::chat::UnsignedMessage;
use crate::moderation::{Ban, Moderator};

#[derive(Deserialize, Serialize)]
pub enum MessageType {
    Unsigned(UnsignedMessage),
    Ban(Ban),
    Mod(Moderator),
}

/// GossipSub Chat Message.
#[derive(Deserialize, Serialize)]
pub struct Message {
    pub msg_type: MessageType,

    /// Link to signed message.
    pub origin: IPLDLink,
}
