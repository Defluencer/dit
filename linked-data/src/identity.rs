use crate::IPLDLink;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Identity {
    /// Your choosen name.
    pub display_name: String,

    /// Link to your avatar. egg an image.
    pub avatar: IPLDLink,
}
