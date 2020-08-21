use crate::hash_timecode::IPLDLink;

use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct LiveNode {
    pub current: IPLDLink,
    pub previous: Option<IPLDLink>,
}

// egg ../<StreamHash>/timecode/hours/0/minutes/36/seconds/12/variants/1080p60/.. => video blocks

#[derive(Serialize, Debug)]
pub struct VariantsNode {
    pub variants: HashMap<String, IPLDLink>,
}
