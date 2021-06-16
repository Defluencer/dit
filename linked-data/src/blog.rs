use crate::IPLDLink;

use serde::{Deserialize, Serialize};

/// Twitter style short message in raw text.
#[derive(Serialize, Deserialize, Debug)]
pub struct ShortMessageNode {
    pub message: String, //TODO switch to text with more characters.
}

//TODO Blog/Forum style long message with formatting.

/// A new discussion topic. Sould always be crypto-signed.
#[derive(Serialize, Deserialize, Debug)]
pub struct TopicStartNode {
    /// GossipSub Topic used to broadcast replies.
    pub topic: String,
    /// Link to the content.
    pub content: IPLDLink,
}

/// A reply to a previous message. Sould always be crypto-signed.
#[derive(Serialize, Deserialize, Debug)]
pub struct ReplyNode {
    /// Link to the signed message.
    pub origin: IPLDLink,
    /// Link to the content.
    pub content: IPLDLink,
}

/// Map node link to each other to form a tree that represent the flow of the conversation.
#[derive(Serialize, Deserialize, Debug)]
pub struct MapNode {
    /// Link to a signed topic start or a reply.
    pub msg_link: IPLDLink,
    /// Links to map node of all replies.
    pub reply_links: Vec<IPLDLink>,
}
