use crate::hash_timecode::IPLDLink;

use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct VODNode {
    pub next: Option<IPLDLink>, // ../<StreamHash>/start/next/.. => second node
    pub current: IPLDLink,      // ../<StreamHash>/start/current/.. => first node
    pub previous: Option<IPLDLink>, // ../<StreamHash>/start/previous/.. => null
}

#[derive(Serialize, Debug)]
pub struct LiveNode {
    pub current: IPLDLink,
    pub previous: Option<IPLDLink>,
}

#[derive(Serialize, Debug)]
pub struct VariantsNode {
    pub variants: HashMap<String, IPLDLink>, // ../current/variants/1080p60/.. => video blocks
}

pub enum StreamVariants {
    Stream1080p60,
    Stream720p60,
    Stream720p30,
    Stream480p30,
}
