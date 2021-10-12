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
    //Could use different indexing method. chrono, keywords, etc...
}

/// Comment metadata and text.
/// Must be crypto-signed to prove authenticity.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Comment {
    /// Timestamp at the time of publication in Unix time.
    pub timestamp: u64,

    /// Link to the author's beacon.
    pub author: IPLDLink,

    /// Link to the content being commented on.
    pub origin: IPLDLink,

    /// Text as content of the comment.
    pub comment: String,
}

impl Comment {
    pub fn create(author: Cid, origin: Cid, comment: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        Self {
            timestamp,
            author: author.into(),
            origin: origin.into(),
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
