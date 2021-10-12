use std::collections::HashMap;

use crate::blog::{FullPost, MicroPost};
use crate::comments::Commentary;
use crate::identity::Identity;
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

/// Identity, Media & Comments Cache
#[derive(Debug, PartialEq, Clone)]
pub struct ContentCache {
    /// Comments CIDs
    comments: Vec<Cid>,

    /// Comment index mapped to beacon index.
    comment_to_beacon: HashMap<usize, usize>,

    /// Beacons CIDs.
    beacons: Vec<Cid>,

    /// Beacon index mapped to name & avatar index.
    beacon_to_identity: HashMap<usize, usize>,

    /// Display names.
    names: Vec<String>,

    /// Links to avatars.
    avatars: Vec<Cid>,

    /// Comment index mapped to media index.
    comment_to_media: HashMap<usize, usize>,

    /// Media CIDs.
    media_content: Vec<Cid>,

    /// Media index mapped to beacon index.
    media_to_beacon: HashMap<usize, usize>,
}

impl ContentCache {
    pub fn create() -> Self {
        Self {
            comments: Vec::with_capacity(100),
            comment_to_beacon: HashMap::with_capacity(100),
            beacons: Vec::with_capacity(100),
            beacon_to_identity: HashMap::with_capacity(100),
            names: Vec::with_capacity(100),
            avatars: Vec::with_capacity(100),
            comment_to_media: HashMap::with_capacity(100),
            media_content: Vec::with_capacity(100),
            media_to_beacon: HashMap::with_capacity(100),
        }
    }

    /// Idempotent way to add a user's identity.
    pub fn insert_identity(&mut self, beacon: Cid, identity: Identity) {
        let beacon_idx = match self.beacons.iter().position(|item| *item == beacon) {
            Some(idx) => idx,
            None => {
                let idx = self.beacons.len();
                self.beacons.push(beacon);

                idx
            }
        };

        match self.beacon_to_identity.get(&beacon_idx) {
            Some(name_idx) => {
                self.names[*name_idx] = identity.display_name;
                self.avatars[*name_idx] = identity.avatar.link;
            }
            None => {
                let name_idx = self.names.len();

                self.names.push(identity.display_name);
                self.avatars.push(identity.avatar.link);

                self.beacon_to_identity.insert(beacon_idx, name_idx);
            }
        }
    }

    /// Idempotent way to add user media content.
    pub fn insert_media_content(&mut self, beacon: Cid, feed: FeedAnchor) {
        let beacon_idx = match self.beacons.iter().position(|item| *item == beacon) {
            Some(idx) => idx,
            None => {
                let idx = self.beacons.len();
                self.beacons.push(beacon);

                idx
            }
        };

        for ipld in feed.content.into_iter() {
            if !self.media_content.contains(&ipld.link) {
                let idx = self.media_content.len();

                self.media_content.push(ipld.link);

                self.media_to_beacon.insert(idx, beacon_idx);
            }
        }
    }

    pub fn iter_media_content(&self) -> impl Iterator<Item = &Cid> {
        self.media_content.iter()
    }

    pub fn media_content_author(&self, media: &Cid) -> Option<&str> {
        let media_idx = self.media_content.iter().position(|item| *item == *media)?;

        let beacon_idx = self.media_to_beacon.get(&media_idx)?;

        let name_idx = self.beacon_to_identity.get(&beacon_idx)?;

        let name = self.names.get(*name_idx)?;

        Some(name)
    }

    /// Idempotent way to add user comments.
    pub fn insert_comments(&mut self, beacon: Cid, commentary: Commentary) {
        let beacon_idx = match self.beacons.iter().position(|item| *item == beacon) {
            Some(idx) => idx,
            None => {
                let idx = self.beacons.len();
                self.beacons.push(beacon);

                idx
            }
        };

        for (media_cid, comments) in commentary.comments.into_iter() {
            let media_idx = match self
                .media_content
                .iter()
                .position(|item| *item == media_cid)
            {
                Some(idx) => idx,
                None => {
                    let idx = self.media_content.len();

                    self.media_content.push(media_cid);

                    idx
                }
            };

            for comment in comments.into_iter() {
                if !self.comments.contains(&comment.link) {
                    let comment_idx = self.comments.len();

                    self.comments.push(comment.link);

                    self.comment_to_beacon.insert(comment_idx, beacon_idx);

                    self.comment_to_media.insert(comment_idx, media_idx);
                }
            }
        }
    }

    pub fn iter_comments(&self, media: &Cid) -> Option<impl Iterator<Item = &Cid>> {
        let media_idx = self.media_content.iter().position(|item| *item == *media)?;

        let iterator = self
            .comment_to_media
            .iter()
            .filter_map(move |(comment_idx, idx)| {
                if *idx == media_idx {
                    self.comments.get(*comment_idx)
                } else {
                    None
                }
            });

        Some(iterator)
    }

    pub fn comment_author(&self, comment: &Cid) -> Option<&str> {
        let comment_idx = self.comments.iter().position(|item| *item == *comment)?;

        let beacon_idx = self.comment_to_beacon.get(&comment_idx)?;

        let name_idx = self.beacon_to_identity.get(&beacon_idx)?;

        let name = self.names.get(*name_idx)?;

        Some(name)
    }

    pub fn comments_count(&self, media: &Cid) -> usize {
        let media_idx = match self.media_content.iter().position(|item| *item == *media) {
            Some(idx) => idx,
            None => return 0,
        };

        self.comment_to_media.values().fold(0, |count, idx| {
            if *idx == media_idx {
                count + 1
            } else {
                count + 0
            }
        })
    }
}
