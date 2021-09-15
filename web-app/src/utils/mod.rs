mod ema;
mod ipfs;
mod local_storage;
mod markdown;
mod web3;

pub use self::web3::Web3Service;
pub use ema::ExponentialMovingAverage;
pub use ipfs::{IpfsService, BRAVE_URI, DEFAULT_URI};
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
