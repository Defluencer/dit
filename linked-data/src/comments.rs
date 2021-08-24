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

#[derive(Debug, PartialEq, Clone)]
pub struct CommentCache {
    origin_indexing: HashMap<Cid, Vec<usize>>, //content cid mapped to indices

    links: Vec<IPLDLink>,
    names: Vec<usize>, //indices into name table

    name_table: Vec<String>,

    temp: Vec<usize>,
}

impl CommentCache {
    pub fn create() -> Self {
        Self {
            origin_indexing: HashMap::with_capacity(10),
            links: Vec::with_capacity(100),
            names: Vec::with_capacity(100),
            name_table: Vec::with_capacity(10),
            temp: Vec::with_capacity(0),
        }
    }

    pub fn insert(&mut self, name: String, commentary: Commentary) {
        let name_table_idx = match self.name_table.iter().position(|item| *item == name) {
            Some(idx) => idx,
            None => {
                let idx = self.name_table.len();
                self.name_table.push(name);

                idx
            }
        };

        for (key, value) in commentary.comments.into_iter() {
            let start_idx = self.links.len(); // inclusive
            self.links.extend(value);
            let end_idx = self.links.len(); // exclusive

            let iter = std::iter::repeat(name_table_idx).take(end_idx - start_idx);
            self.names.extend(iter);

            let indices = (start_idx..end_idx).collect();

            match self.origin_indexing.get_mut(&key) {
                Some(vec) => vec.extend(indices),
                None => {
                    self.origin_indexing.insert(key, indices);
                }
            }
        }
    }

    pub fn iter_per_origin(&self, origin: &Cid) -> impl Iterator<Item = &IPLDLink> {
        let idx = match self.origin_indexing.get(origin) {
            Some(vec) => vec,
            None => &self.temp,
        };

        idx.iter().map(move |idx| &self.links[*idx])
    }

    pub fn get_comment_name(&self, cid: &Cid) -> Option<&str> {
        let idx = match self.links.iter().position(|item| item.link == *cid) {
            Some(idx) => idx,
            None => return None,
        };

        let name = &self.name_table[self.names[idx]];

        Some(name)
    }

    pub fn get_comment_count(&self, origin: &Cid) -> usize {
        match self.origin_indexing.get(origin) {
            Some(vec) => vec.len(),
            None => 0,
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

        let json = serde_json::to_string(&old_comments).expect("Serialize");
        println!("{}", json);

        let new_comments = serde_json::from_str(&json).expect("Deserialize");
        println!("{:?}", new_comments);

        assert_eq!(old_comments, new_comments);
    }
}
