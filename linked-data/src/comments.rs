use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// List of all comment lists.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CommentsAnchor {
    /// In sync with content feed. Indexes are the same.
    pub links: Vec<IPLDLink>,
}

/// List of comments of some content.
/// Recursive Pin.
#[derive(Serialize, Deserialize, Debug)]
pub struct Comments {
    comments: Vec<IPLDLink>,
}

/// A comment signaling node. Sould always be crypto-signed.
#[derive(Serialize, Deserialize, Debug)]
pub struct CommentLink {
    /// Link to the original content.
    pub origin: IPLDLink,

    /// Link to the comment being replied to.
    pub reply: Option<IPLDLink>,

    /// Link to the comment content itself.
    pub comment: IPLDLink,
}
