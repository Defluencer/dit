use std::collections::HashSet;

use crate::IPLDLink;

use serde::{Deserialize, Serialize};

use either::Either;

/// List of all your friends.
/// Direct Pin.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Friendlies {
    pub friends: HashSet<Friend>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Friend {
    /// Domain name on the Ethereum Name Service or
    /// Link to friend's beacon.
    #[serde(with = "either::serde_untagged")]
    pub friend: Either<String, IPLDLink>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cid::Cid;

    #[test]
    fn serde_test() {
        let mut old_friends = Friendlies {
            friends: HashSet::with_capacity(2),
        };

        old_friends.friends.insert(Friend {
            friend: Either::Left("friend1".to_owned()),
        });

        old_friends.friends.insert(Friend {
            friend: Either::Right(Cid::default().into()),
        });

        let json = serde_json::to_string_pretty(&old_friends).expect("Cannot Serialize");
        println!("{}", json);

        let new_friends = serde_json::from_str(&json).expect("Cannot Deserialize");
        println!("{:?}", new_friends);

        assert_eq!(old_friends, new_friends);
    }
}
