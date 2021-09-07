use std::collections::HashMap;

use crate::blog::{FullPost, MicroPost};
use crate::comments::Commentary;
use crate::video::VideoMetadata;
use crate::IPLDLink;

use serde::{Deserialize, Serialize};

use cid::Cid;

/// Content feed in chronological order.
/// Direct pin.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct FeedAnchor {
    /// List of links to content ordered from oldest to newest.
    pub content: Vec<IPLDLink>,
}

#[derive(Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum Media {
    Statement(MicroPost),
    Blog(FullPost),
    Video(VideoMetadata),
}

impl Media {
    pub fn timestamp(&self) -> u64 {
        match self {
            Media::Statement(metadata) => metadata.timestamp,
            Media::Blog(metadata) => metadata.timestamp,
            Media::Video(metadata) => metadata.timestamp,
        }
    }
}

/// Media & Comments Cache
#[derive(Debug, PartialEq, Clone)]
pub struct ContentCache {
    //content cid mapped to name table index and indices into comments
    origin_indexing: HashMap<Cid, (Option<usize>, Vec<usize>)>,

    comments: Vec<IPLDLink>, //sync
    names: Vec<usize>,       //sync, also indices into name table

    name_table: Vec<String>,

    temp: Vec<usize>,
}

impl ContentCache {
    pub fn create() -> Self {
        Self {
            origin_indexing: HashMap::with_capacity(100),
            comments: Vec::with_capacity(100),
            names: Vec::with_capacity(100),
            name_table: Vec::with_capacity(10),
            temp: Vec::with_capacity(0),
        }
    }

    pub fn insert_comments(&mut self, name: String, commentary: Commentary) {
        let name_table_idx = match self.name_table.iter().position(|item| *item == name) {
            Some(idx) => idx,
            None => {
                let idx = self.name_table.len();
                self.name_table.push(name);

                idx
            }
        };

        for (key, value) in commentary.comments.into_iter() {
            let start_idx = self.comments.len(); // inclusive
            self.comments.extend(value);
            let end_idx = self.comments.len(); // exclusive

            let iter = std::iter::repeat(name_table_idx).take(end_idx - start_idx);
            self.names.extend(iter);

            let indices = (start_idx..end_idx).collect();

            match self.origin_indexing.get_mut(&key) {
                Some((_, vec)) => vec.extend(indices),
                None => {
                    self.origin_indexing.insert(key, (None, indices));
                }
            }
        }
    }

    pub fn insert_content(&mut self, name: String, feed: FeedAnchor) {
        let name_table_idx = match self.name_table.iter().position(|item| *item == name) {
            Some(idx) => idx,
            None => {
                let idx = self.name_table.len();
                self.name_table.push(name);

                idx
            }
        };

        for ipld in feed.content.into_iter() {
            match self.origin_indexing.get_mut(&ipld.link) {
                Some((idx, _)) => *idx = Some(name_table_idx),
                None => {
                    self.origin_indexing
                        .insert(ipld.link, (Some(name_table_idx), vec![]));
                }
            }
        }
    }

    pub fn iter_comments(&self, origin: &Cid) -> impl Iterator<Item = &IPLDLink> {
        let idx = match self.origin_indexing.get(origin) {
            Some((_, vec)) => vec,
            None => &self.temp,
        };

        idx.iter().map(move |idx| &self.comments[*idx])
    }

    pub fn iter_content(&self) -> impl Iterator<Item = &Cid> {
        self.origin_indexing.keys()
    }

    pub fn get_comment_author(&self, cid: &Cid) -> Option<&str> {
        let idx = match self.comments.iter().position(|item| item.link == *cid) {
            Some(idx) => idx,
            None => return None,
        };

        let name = &self.name_table[self.names[idx]];

        Some(name)
    }

    pub fn get_content_author(&self, cid: &Cid) -> Option<&str> {
        let (idx, _) = self.origin_indexing.get(cid)?;

        let idx = idx.as_ref()?;

        let name = &self.name_table[*idx];

        Some(name)
    }

    pub fn get_comments_count(&self, origin: &Cid) -> usize {
        match self.origin_indexing.get(origin) {
            Some((_, vec)) => vec.len(),
            None => 0,
        }
    }
}
