use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// List of all your friends.
/// Recursive Pin.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Friendlies {
    /// Domains names own by friends on the Ethereum Name Service.
    pub ens_domains: Vec<String>,

    /// Links to friends beacons
    pub beacons: Vec<IPLDLink>,
}
