use crate::IPLDLink;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Feed {
    pub content: Vec<IPLDLink>,
}
