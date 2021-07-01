use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Metadata for a long blog post.
#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct FullPost {
    /// The title of this blog post
    pub title: String,

    /// Link to thumbnail image.
    pub image: IPLDLink,

    /// Link to markdown file
    pub content: IPLDLink,

    /// Timestamp at the time of publication in Unix time.
    pub timestamp: u64,
}

/// A micro blog post (Twitter-sytle).
#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct MicroPost {
    pub content: String,

    /// Timestamp at the time of publication in Unix time.
    pub timestamp: u64,
}
