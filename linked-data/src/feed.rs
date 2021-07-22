use crate::blog::{FullPost, MicroPost};
use crate::video::VideoMetadata;
use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Content feed in chronological order.
/// Direct pin.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct FeedAnchor {
    /// In sync with comments. Indexes are the same.
    pub content: Vec<IPLDLink>,
}

#[derive(Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Media {
    Statement(MicroPost),
    Blog(FullPost),
    Video(VideoMetadata),
}

impl Default for Media {
    fn default() -> Self {
        Self::Statement(MicroPost::default())
    }
}
