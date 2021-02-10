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

/// Translate total number of seconds to timecode.
pub fn seconds_to_timecode(seconds: f64) -> (u8, u8, u8) {
    let rem_seconds = seconds.round();

    let hours = (rem_seconds / 3600.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(3600.0);

    let minutes = (rem_seconds / 60.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(60.0);

    let seconds = rem_seconds as u8;

    (hours, minutes, seconds)
}
