use ipfs_api::response::{KeyListResponse, KeyPair};

pub fn search_keypairs(name: &str, res: KeyListResponse) -> Option<KeyPair> {
    for keypair in res.keys {
        if keypair.name == name {
            return Some(keypair);
        }
    }

    None
}
