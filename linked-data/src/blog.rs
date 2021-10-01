use crate::IPLDLink;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use cid::Cid;

/// A micro blog post (Twitter-sytle).
/// Recursive pin.
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct MicroPost {
    /// Timestamp at the time of publication in Unix time.
    pub timestamp: u64,

    /// Link to the author's beacon.
    pub author: IPLDLink,

    /// Text as content of the blog post.
    pub content: String,
}

impl MicroPost {
    pub fn create(author: Cid, content: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        Self {
            timestamp,
            author: author.into(),
            content,
        }
    }

    pub fn update(&mut self, content: String) {
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        self.content = content;
    }
}

/// Metadata for a long blog post.
/// Recursive pin.
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct FullPost {
    /// Timestamp at the time of publication in Unix time.
    pub timestamp: u64,

    /// Link to the author's beacon.
    pub author: IPLDLink,

    /// Link to markdown file
    pub content: IPLDLink,

    /// Link to thumbnail image.
    pub image: IPLDLink,

    /// The title of this blog post
    pub title: String,
}

impl FullPost {
    pub fn create(title: String, image: Cid, markdown: Cid, author: Cid) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        Self {
            timestamp,
            author: author.into(),
            title,
            image: image.into(),
            content: markdown.into(),
        }
    }

    pub fn update(&mut self, title: Option<String>, image: Option<Cid>, video: Option<Cid>) {
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        if let Some(title) = title {
            self.title = title;
        }

        if let Some(img) = image {
            self.image = img.into();
        }

        if let Some(vid) = video {
            self.content = vid.into();
        }
    }
}
