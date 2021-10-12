use crate::IPNSAddress;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

/// Static links to content.
/// Direct pin.
#[serde_as]
#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Clone)]
pub struct Beacon {
    /// Link to avatar, name, etc...
    #[serde_as(as = "DisplayFromStr")]
    pub identity: IPNSAddress,

    /// Link to list of content metadata.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub content_feed: Option<IPNSAddress>,

    /// Link to list of comments.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub comments: Option<IPNSAddress>,

    /// Link to topics and Peer Id for streming live.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub live: Option<IPNSAddress>,

    /// Link to list of your friend's beacons.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub friends: Option<IPNSAddress>,

    /// Link to all chat banned addresses.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bans: Option<IPNSAddress>,

    /// Link to all chat moderator addresses.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub mods: Option<IPNSAddress>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cid::Cid;
    use std::str::FromStr;

    #[test]
    fn serde_test() {
        let cid =
            Cid::from_str("bafyreibjo4xmgaevkgud7mbifn3dzp4v4lyaui4yvqp3f2bqwtxcjrdqg4").unwrap();

        let old_beacon = Beacon {
            identity: cid,
            content_feed: Some(Cid::default()),
            comments: None,
            friends: None,
            live: None,
            bans: None,
            mods: None,
        };

        let json = serde_json::to_string_pretty(&old_beacon).expect("Cannot serialize");
        println!("{}", json);

        let new_beacon = serde_json::from_str(&json).expect("Cannot Deserialize");
        println!("{:?}", new_beacon);

        assert_eq!(old_beacon, new_beacon);
    }
}
