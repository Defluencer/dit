use crate::chat::Address;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ban {
    pub address: Address,
    pub peer_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vip {
    pub vip: Address,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Moderator {
    #[serde(rename = "mod")]
    pub moderator: Address,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Bans {
    pub banned: HashSet<Address>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Vips {
    pub vips: HashSet<Address>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Moderators {
    pub mods: HashSet<Address>,
}
