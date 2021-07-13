use crate::{Address, PeerId};

use std::collections::HashMap;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use cid::Cid;

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
/// Direct pin.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Bans {
    pub banned: HashSet<Address>,
}

/// List of moderators.
/// Direct pin.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Moderators {
    pub mods: HashSet<Address>,
}

/// Local cache of who is verified and/or banned.
pub struct ChatModerationCache {
    verified: HashMap<PeerId, usize>, // Map peer IDs to indices.

    peers: Vec<PeerId>,      // sync
    origins: Vec<Cid>,       // sync
    addresses: Vec<Address>, // sync
    names: Vec<String>,      // sync

    ban_index: usize, // Lower than this users are banned.
}

impl ChatModerationCache {
    pub fn new(capacity: usize, name_cap: usize) -> Self {
        Self {
            verified: HashMap::with_capacity(capacity),

            peers: Vec::with_capacity(capacity),
            origins: Vec::with_capacity(capacity),
            addresses: Vec::with_capacity(capacity),
            names: Vec::with_capacity(name_cap),

            ban_index: 0,
        }
    }

    /// Check if this peer is banned.
    pub fn is_banned(&self, peer: &str) -> bool {
        let index = match self.verified.get(peer) {
            Some(i) => *i,
            None => return false,
        };

        index < self.ban_index
    }

    /// Check if this peer is verified.
    pub fn is_verified(&self, peer: &str, origin: &Cid) -> bool {
        let index = match self.verified.get(peer) {
            Some(i) => *i,
            None => return false,
        };

        origin == &self.origins[index]
    }

    pub fn get_address(&self, peer: &str) -> Option<&Address> {
        let index = self.verified.get(peer)?;

        let address = self.addresses.get(*index)?;

        Some(address)
    }

    pub fn get_name(&self, peer: &str) -> Option<&str> {
        let index = self.verified.get(peer)?;

        let name = self.names.get(*index)?;

        Some(name)
    }

    /// Add as verified user.
    pub fn add_peer(&mut self, peer: &str, cid: Cid, addrs: Address, name: Option<String>) {
        if self.verified.contains_key(peer) {
            return;
        }

        let index = self.peers.len();

        self.peers.push(peer.to_owned());
        self.origins.push(cid);
        self.addresses.push(addrs);

        if let Some(name) = name {
            self.names.push(name);
        }

        self.verified.insert(peer.to_owned(), index);
    }

    pub fn ban_peer(&mut self, peer: &str) {
        let i = match self.verified.get(peer) {
            Some(i) => *i,
            None => return,
        };

        if i < self.ban_index {
            return;
        }

        if i == self.ban_index {
            self.ban_index += 1;
            return;
        }

        let last = self.ban_index;

        self.peers.swap(i, last);
        self.origins.swap(i, last);
        self.addresses.swap(i, last);
        self.names.swap(i, last);

        let index = self.verified.get_mut(peer).unwrap();
        *index = last;

        let last_peer = &self.peers[i];
        let index = self.verified.get_mut(last_peer).unwrap();
        *index = i;

        self.ban_index += 1;
    }
}
