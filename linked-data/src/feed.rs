use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Content feed in chronological order.
/// Direct pin.
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Feed {
    pub content: Vec<IPLDLink>,
}
