use std::collections::HashSet;

use crate::IPLDLink;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct ChatIdentity {
    #[serde(rename = "key")]
    pub public_key: String,
}

/// Chat message optionaly signed with some form of private key
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub identity: ChatIdentity,

    pub signature: String,

    pub data: ChatContent,
}

/// User name, message and a link to VideoNode as timestamp
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatContent {
    pub name: String,

    pub message: String,

    pub timestamp: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Blacklist {
    pub blacklist: HashSet<ChatIdentity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Whitelist {
    pub whitelist: HashSet<ChatIdentity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Moderators {
    pub mods: HashSet<ChatIdentity>,
}
