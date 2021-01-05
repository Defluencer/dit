use std::sync::{Arc, RwLock};

use web_sys::SourceBuffer;

pub type Tracks = Arc<RwLock<Vec<Track>>>;

pub struct Track {
    pub level: usize,
    pub quality: String,
    pub codec: String,
    pub source_buffer: SourceBuffer,
    //pub bandwidth: usize,
    //pub init_seg_cid: Cid,
}
