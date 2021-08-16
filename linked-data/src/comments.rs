use crate::IPLDLink;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Metadata of all your comments.
/// Direct Pin.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Commentary {
    /// Content cids mapped to lists of links to signed comments ordered from oldest to newest.
    pub metadata: HashMap<String, Vec<IPLDLink>>,
}

impl Commentary {
    pub fn merge(&mut self, other: Self) {
        for (cid, mut links) in other.metadata.into_iter() {
            match self.metadata.get_mut(&cid) {
                Some(vec) => {
                    vec.append(&mut links);
                }
                None => {
                    self.metadata.insert(cid, links);
                }
            }
        }
    }
}

/// A comment. Must be crypto-signed.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    /// Timestamp at the time of publication in Unix time.
    pub timestamp: u64,

    /// Link to the original content.
    pub origin: IPLDLink,

    /// Link to the comment being replied to.
    pub reply: Option<IPLDLink>,

    pub comment: String,
}

impl Comment {
    pub fn create(origin: IPLDLink, reply: Option<IPLDLink>, comment: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        Self {
            timestamp,
            origin,
            reply,
            comment,
        }
    }
}

//TODO build a comments database
//search by content cid
//deduplication
//easy merging
