use std::collections::HashMap;

use crate::IPLDLink;

use serde::{Deserialize, Serialize};

// Comments are linked to content, if content change the comments won't follow to the new version.

/// List of all comments.
#[derive(Serialize, Deserialize, Debug)]
pub struct Comments {
    /// The keys are links to content.
    /// The values are links to conversation tree leaf CIDs.
    pub comments: HashMap<IPLDLink, Vec<IPLDLink>>,
}

/// A comment signaling node. Sould always be crypto-signed.
#[derive(Serialize, Deserialize, Debug)]
pub struct CommentNode {
    /// Link to the original content or another comment.
    pub origin: IPLDLink,

    /// Link to the comment content itself.
    pub comment: IPLDLink,
}
