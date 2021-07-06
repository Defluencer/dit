use crate::IPLDLink;

use serde::{Deserialize, Serialize};

use cid::Cid;

#[derive(Serialize, Deserialize, Debug)]
struct Create {
    create: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug)]
struct Update {
    /// the new content
    create: IPLDLink,
    /// the content that was updated
    update: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug)]
struct Delete {
    delete: IPLDLink,
}

#[derive(Serialize, Deserialize, Debug)]
enum FeedAction {
    Create(Create),
    Update(Update),
    Delete(Delete),
}

/// Content feed node links to other nodes forming an append-only log.
/// Do NOT pin recursively!
#[derive(Serialize, Deserialize, Debug)]
pub struct Feed {
    action: FeedAction,
    /// Link to previous content feed element.
    previous: IPLDLink,
}

impl Feed {
    pub fn add(create: Cid, previous: Cid) -> Self {
        Self {
            action: FeedAction::Create(Create {
                create: create.into(),
            }),
            previous: previous.into(),
        }
    }

    pub fn update(create: Cid, update: Cid, previous: Cid) -> Self {
        Self {
            action: FeedAction::Update(Update {
                create: create.into(),
                update: update.into(),
            }),
            previous: previous.into(),
        }
    }

    pub fn delete(delete: Cid, previous: Cid) -> Self {
        Self {
            action: FeedAction::Delete(Delete {
                delete: delete.into(),
            }),
            previous: previous.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FeedStart {}
