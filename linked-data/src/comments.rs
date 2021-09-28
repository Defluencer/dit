use crate::IPLDLink;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use cid::Cid;

/// Metadata of all your comments.
/// Direct Pin.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct Commentary {
    /// Content cids mapped to lists of links to comments ordered from oldest to newest.
    #[serde_as(as = "HashMap<DisplayFromStr, Vec<_>>")]
    pub comments: HashMap<Cid, Vec<IPLDLink>>,
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
    use std::str::FromStr;

    #[test]
    fn serde_test() {
        let mut old_comments = Commentary {
            comments: HashMap::with_capacity(2),
        };

        let cid =
            Cid::from_str("bafyreibjo4xmgaevkgud7mbifn3dzp4v4lyaui4yvqp3f2bqwtxcjrdqg4").unwrap();

        old_comments
            .comments
            .insert(cid, vec![Cid::default().into()]);
        old_comments
            .comments
            .insert(cid, vec![Cid::default().into()]);

        let json = serde_json::to_string_pretty(&old_comments).expect("Cannot Serialize");
        println!("{}", json);

        let new_comments = serde_json::from_str(&json).expect("Cannot Deserialize");
        println!("{:?}", new_comments);

        assert_eq!(old_comments, new_comments);
    }
}
