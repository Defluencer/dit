//use crate::IPLDLink;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Blacklist {
    pub eth: HashSet<[u8; 20]>,
    pub peer_ids: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Whitelist {
    pub eth: HashSet<[u8; 20]>,
    pub peer_ids: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Moderators {
    pub mods: HashSet<[u8; 20]>,
}

//TODO Timeout/ban messages
