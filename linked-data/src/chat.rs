use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Unsigned chat message with origin.
#[derive(Serialize, Deserialize, Debug)]
pub struct UnsignedMessage {
    pub message: String,

    /// Link to signed message.
    pub origin: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub name: String,

    pub peer_id: String,
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

use secp256k1::recover;
use secp256k1::{Message, RecoveryId, Signature};

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
