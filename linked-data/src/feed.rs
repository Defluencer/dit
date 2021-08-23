use crate::blog::{FullPost, MicroPost};
use crate::video::VideoMetadata;
use crate::IPLDLink;

use serde::{Deserialize, Serialize};

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
