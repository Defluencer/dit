use crate::IPLDLink;

use serde::{Deserialize, Serialize};

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

use web3::signing::{keccak256, recover};

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

        //https://docs.rs/web3/0.15.0/web3/signing/fn.recover.html
        //The actual signature is the first 64 bytes.
        //The last byte is the recovery id.
        let res = match recover(&hash, &self.signature[0..64], self.signature[64] as i32) {
            Ok(data) => data,
            Err(_) => return false,
        };

        res.to_fixed_bytes() == self.address
    }
}
