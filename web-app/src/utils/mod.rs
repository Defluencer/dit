mod ema;
mod ipfs;
mod local_storage;
mod markdown;
mod web3;

pub use self::web3::Web3Service;
pub use ema::ExponentialMovingAverage;
pub use ipfs::{IpfsService, DEFAULT_URI};
pub use local_storage::LocalStorage;
pub use markdown::render_markdown;

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

/// Unix time in total number of seconds to date time string.
pub fn timestamp_to_datetime(seconds: u64) -> String {
    use chrono::{DateTime, Local, TimeZone, Utc};

    let d_t_unix = Utc.timestamp(seconds as i64, 0);

    let local_d_t = DateTime::<Local>::from(d_t_unix);

    local_d_t.format("%Y-%m-%d %H:%M:%S").to_string()
}
