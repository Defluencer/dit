use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use secp256k1::recover;
use secp256k1::{Message, RecoveryId, Signature};

use cid::Cid;

/// Ethereum address
pub type Address = [u8; 20];

/// GossipSub Peer ID
pub type PeerId = String;

/// Unsigned chat message with verifiable origin.
#[derive(Serialize, Deserialize, Debug)]
pub struct UnsignedMessage {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub name: String,

    pub peer: PeerId,
}

/// Crypto-signed message origin.
#[derive(Serialize, Deserialize, Debug)]
pub struct SignedMessage {
    pub address: Address,

    pub data: Content,

    pub signature: Vec<u8>, // Should be [u8; 65] but serde can't deal with big arrays.
}

impl SignedMessage {
    pub fn verify(&self) -> bool {
        if self.signature.len() != 65 {
            return false;
        }

        let message = serde_json::to_vec(&self.data).expect("Cannot Serialize");

        let mut eth_message =
            format!("\x19Ethereum Signed Message:\n{}", message.len()).into_bytes();
        eth_message.extend_from_slice(&message);

        let hash = keccak256(&eth_message);

        let msg = Message::parse_slice(&hash).expect("Invalid Message");
        let sig = Signature::parse_slice(&self.signature[0..64]).expect("Invalid Signature");
        let rec_id = RecoveryId::parse_rpc(self.signature[64]).expect("Invalid Recovery Id");

        let public_key = match recover(&msg, &sig, &rec_id) {
            Ok(data) => data.serialize(),
            Err(_) => return false,
        };

        // The public key returned is 65 bytes long, that is because it is prefixed by `0x04` to indicate an uncompressed public key.
        let hash = keccak256(&public_key[1..]);

        // The public address is defined as the low 20 bytes of the keccak hash of the public key.
        hash[12..] == self.address
    }
}

/// Compute the Keccak-256 hash of input bytes.
fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}

pub struct LocalModerationDB {
    verified: HashMap<PeerId, usize>, // Map peer IDs to indices.

    peers: Vec<PeerId>,      // sync
    origins: Vec<Cid>,       // sync
    addresses: Vec<Address>, // sync
    names: Vec<String>,      // sync

    ban_index: usize, // Lower than this users are banned.
}

impl LocalModerationDB {
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
    pub fn verified(&self, peer: &str, origin: &Cid) -> bool {
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
