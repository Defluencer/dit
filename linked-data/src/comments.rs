use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// List of all comment lists.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CommentsAnchor {
    /// Links to list of comments.
    pub links: Vec<IPLDLink>, // In sync with content feed. Indexes are the same.
}

/// List of comments of some content.
/// Recursive Pin.
/// Must be unpinned when updating the content otherwise it will recursive pin the old content.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Comments {
    pub list: Vec<IPLDLink>,
}

/// A comment signaling node. Must be crypto-signed.
#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    /// Link to the original content.
    pub origin: IPLDLink,

    /// Link to the comment being replied to.
    pub reply: Option<IPLDLink>,

    pub comment: String,
}

// To display comments iterate in reverse but skip replies to other comments, save for next step
// Display replies and repeat until no more comments.
