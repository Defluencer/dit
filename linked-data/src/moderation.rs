use crate::{Address, PeerId};

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Message to ban/unban a user.
#[derive(Serialize, Deserialize, Debug)]
pub struct Ban {
    pub address: Address,
    pub peer_id: PeerId,
}

/// Message to mod/unmod a user.
#[derive(Serialize, Deserialize, Debug)]
pub struct Moderator {
    #[serde(rename = "mod")]
    pub moderator: Address,
}

/// List of banned users.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Bans {
    pub banned: HashSet<Address>,
}

/// List of moderators.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Moderators {
    pub mods: HashSet<Address>,
}
