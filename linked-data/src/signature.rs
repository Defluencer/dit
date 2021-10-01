use crate::{keccak256, Address};

use serde::{Deserialize, Serialize};

use libsecp256k1::recover;
use libsecp256k1::{Message, RecoveryId, Signature};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Generic crypto-signed message.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SignedMessage<T>
where
    T: Serialize,
{
    pub address: Address,

    pub data: T,

    pub signature: Vec<u8>, // Should be [u8; 65] but serde can't deal with big arrays.
}

impl<T> SignedMessage<T>
where
    T: Serialize,
{
    pub fn verify(&self) -> bool {
        if self.signature.len() != 65 {
            return false;
        }

        let public_key = match self.public_key() {
            Ok(key) => key,
            Err(_) => return false,
        };

        // The public key returned is 65 bytes long, that is because it is prefixed by `0x04` to indicate an uncompressed public key.
        let hash = keccak256(&public_key[1..]);

        // The public address is defined as the low 20 bytes of the keccak hash of the public key.
        hash[12..] == self.address
    }

    fn public_key(&self) -> Result<[u8; 65]> {
        let message = serde_json::to_vec(&self.data)?;

        let mut eth_message =
            format!("\x19Ethereum Signed Message:\n{}", message.len()).into_bytes();
        eth_message.extend_from_slice(&message);

        let hash = keccak256(&eth_message);

        let msg = Message::parse_slice(&hash)?;

        let sig = Signature::parse_standard_slice(&self.signature[0..64])?;

        let rec_id = RecoveryId::parse_rpc(self.signature[64])?;

        let data = recover(&msg, &sig, &rec_id)?;

        Ok(data.serialize())
    }
}
