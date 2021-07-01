use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Content Feed in cronological order.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Feed {
    pub content: Vec<IPLDLink>,
}
