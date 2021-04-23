use crate::IPLDLink;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ChatMessage {
    Signed(SignedMessage),
    Unsigned(UnsignedMessage),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub peer_id: String,

    pub name: String,
}

/// Crypto-signed message origin.
#[derive(Serialize, Deserialize, Debug)]
pub struct SignedMessage {
    /// Ethereum address
    pub address: [u8; 20],

    pub data: Content,

    /// Content crypto-signed with this address.
    pub signature: Vec<u8>,
}

/// Unsigned chat message with origin.
#[derive(Serialize, Deserialize, Debug)]
pub struct UnsignedMessage {
    pub message: String,

    /// Link to signed message.
    pub origin: IPLDLink,
}
