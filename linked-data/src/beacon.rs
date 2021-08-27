use crate::{IPNSLink, PeerId};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Topics {
    pub video: String,
    pub chat: String,
}

/// Mostly static links to content.
/// Direct pin.
#[serde_as]
#[derive(Deserialize, Serialize, Default, Debug, PartialEq, Clone)]
pub struct Beacon {
    /// GossipSub Topics.
    pub topics: Topics,

    /// IPFS Peer ID. Base58btc.
    pub peer_id: PeerId,

    /// Your choosen name.
    pub display_name: String,

    /// Link to list of content metadata.
    #[serde_as(as = "DisplayFromStr")]
    pub content_feed: IPNSLink, //IPNS path -> "/ipns/<hash>"

    /// Link to list of comments.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub comments: Option<IPNSLink>, //IPNS path -> "/ipns/<hash>"

    /// Link to list of your friend's beacons.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub friends: Option<IPNSLink>, //IPNS path -> "/ipns/<hash>"

    /// Link to all banned addresses.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub bans: Option<IPNSLink>, //IPNS path -> "/ipns/<hash>"

    /// Link to all moderator addresses.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub mods: Option<IPNSLink>, //IPNS path -> "/ipns/<hash>"
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
            topics: Topics {
                video: "gdgdgd".to_owned(),
                chat: "dhhdhd".to_owned(),
            },

            peer_id: "uhduhdjhdh".to_owned(),

            display_name: "Name".to_owned(),

            content_feed: cid,

            comments: Some(Cid::default()),

            friends: None,

            bans: None,

            mods: None,
        };

        let json = serde_json::to_string_pretty(&old_beacon).expect("Serialize");
        println!("{}", json);

        let new_beacon = serde_json::from_str(&json).expect("Deserialize");
        println!("{:?}", new_beacon);

        assert_eq!(old_beacon, new_beacon);
    }
}
