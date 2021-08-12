use crate::IPLDLink;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

//Resolve the comment of your friends and the friend of your friends to see their comments.

/// Metadata of all your comments.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Commentary {
    /// Map CID strings to link of signed comments.
    pub metadata: HashMap<String, Vec<IPLDLink>>,
}

/// A comment. Must be crypto-signed.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    /// Link to the original content.
    pub origin: IPLDLink,

    /// Link to the comment being replied to.
    pub reply: Option<IPLDLink>,

    pub comment: String,
}
