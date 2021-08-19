use crate::IPLDLink;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Metadata of all your comments.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Commentary {
    /// Content cids mapped to lists of links to signed comments ordered from oldest to newest.
    pub map: HashMap<String, Vec<IPLDLink>>,
}

impl Commentary {
    pub fn merge(&mut self, other: Self) {
        for (cid, mut links) in other.map.into_iter() {
            match self.map.get_mut(&cid) {
                Some(vec) => {
                    vec.append(&mut links);
                }
                None => {
                    self.map.insert(cid, links);
                }
            }
        }
    }
}

/// Comment metadata and text.
/// Must be crypto-signed to prove authenticity.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use cid::Cid;

    #[test]
    fn serde_test() {
        let mut old_comments = Commentary {
            map: HashMap::with_capacity(2),
        };

        old_comments
            .map
            .insert("CID1".to_owned(), vec![Cid::default().into()]);
        old_comments
            .map
            .insert("CID2".to_owned(), vec![Cid::default().into()]);

        let json = match serde_json::to_string_pretty(&old_comments) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        println!("{}", json);

        let new_comments = match serde_json::from_str(&json) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        println!("{:?}", new_comments);

        assert_eq!(old_comments, new_comments);
    }
}
