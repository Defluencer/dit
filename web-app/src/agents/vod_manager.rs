/* use std::sync::Arc;

use crate::utils::ema::ExponentialMovingAverage;
use crate::utils::ipfs::{cat_and_buffer, ipfs_dag_get_path_async};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use web_sys::{HtmlMediaElement, MediaSource, SourceBuffer, Url, Window};

use yew::services::ConsoleService;

use linked_data::video::{SetupNode, Track, VideoMetadata};

const FORWARD_BUFFER_LENGTH: f64 = 16.0;
const BACK_BUFFER_LENGTH: f64 = 8.0;

const MEDIA_LENGTH: f64 = 4.0;
const MEDIA_LENGTH_MS: f64 = 4000.0;

const SETUP_PATH: &str = "/time/hour/0/minute/0/second/0/video/setup/";

//TODO add types common to live and vod then deduplicate fonctions.

//TODO save setup node instead of Tracks

enum MachineState {
    Load,
    Switch,
    Flush,
    Timeout,
}

struct MediaBuffer {
    video: SourceBuffer,
    audio: SourceBuffer,

    tracks: Vec<Track>,
}

struct Video {
    metadata: VideoMetadata,

    window: Window,
    media_element: Option<HtmlMediaElement>,
    media_source: MediaSource,

    media_buffer: Option<MediaBuffer>,

    state: MachineState,
    level: usize,

    ema: ExponentialMovingAverage,

    handle: i32,
}

pub struct VideoOnDemandManager {
    video: Arc<Video>,

    url: String,
}

impl VideoOnDemandManager {
    /// Ready VOD to link with video element.
    pub fn new(metadata: VideoMetadata) -> Self {
        let window = web_sys::window().expect("Can't get window");

        let ema = ExponentialMovingAverage::new(&window);

        let media_source = MediaSource::new().expect("Can't create media source");

        let url = Url::create_object_url_with_source(&media_source)
            .expect("Can't create url from source");

        let video = Video {
            metadata,

            window,
            media_element: None,
            media_source,

            media_buffer: None,

            state: MachineState::Timeout,
            level: 0,

            ema,

            handle: 0,
        };

        let video = Arc::new(video);

        Self { video, url }
    }

    /// Get video element, register callbacks and set source.
    pub fn link_video(&mut self) {
        let document = self.video.window.document().expect("Can't get document");

        let media_element: HtmlMediaElement = document
            .get_element_by_id("video_player")
            .expect("No element with this Id")
            .dyn_into()
            .expect("Not Media Element");

        self.video.media_element = Some(media_element.clone());

        on_media_source_open(self.video.clone(), &self.video.media_source);

        on_video_seeking(self.video.clone(), &media_element);

        media_element.set_src(&self.url);
    }
}

impl Drop for VideoOnDemandManager {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping VideoOnDemandManager");

        let handle = self.video.handle;

        if handle != 0 {
            self.video.window.clear_interval_with_handle(handle);
        }
    }
}

/// Callback when MediaSource is linked to video element.
fn on_media_source_open(video: Arc<Video>, media_source: &MediaSource) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Source Open");

        video.media_source.set_onsourceopen(None);

        video.media_source.set_duration(video.metadata.duration);

        let future = add_source_buffer(video.clone());

        spawn_local(future);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_source.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));
}

/// Get setup infos, create source buffer then load initialization segment.
async fn add_source_buffer(video: Arc<Video>) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Adding Source Buffer");

    let cid = video.metadata.video.link;

    let setup_node: SetupNode = match ipfs_dag_get_path_async(cid, SETUP_PATH).await {
        Ok(result) => result,
        Err(_) => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Setup Node \n {}",
        &serde_json::to_string_pretty(&setup_node).expect("Can't print")
    ));

    #[cfg(debug_assertions)]
    ConsoleService::info("Listing Tracks");

    let mut audio_buffer = None;
    let mut video_buffer = None;

    for (level, track) in setup_node.tracks.iter().enumerate() {
        if !MediaSource::is_type_supported(&track.codec) {
            ConsoleService::error(&format!("MIME Type {:?} unsupported", &track.codec));
            continue;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Level {} Name {} Codec {} Bandwidth {}",
            level, track.name, track.codec, track.bandwidth
        ));

        if audio_buffer.is_some() && video_buffer.is_some() {
            continue;
        }

        let source_buffer = match video.media_source.add_source_buffer(&track.codec) {
            Ok(sb) => sb,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        if track.name == "audio" {
            audio_buffer = Some(source_buffer);
        } else {
            video_buffer = Some(source_buffer);
        }
    }

    let path = setup_node.tracks[0].initialization_segment.link.to_string();

    on_source_buffer_update_end(video, video_buffer.as_ref().unwrap());

    let media_buffer = MediaBuffer {
        tracks: setup_node.tracks,
        audio: audio_buffer.unwrap(),
        video: video_buffer.unwrap(),
    };

    video.media_buffer = Some(media_buffer);
    video.state = MachineState::Load;

    //cat_and_buffer(path, source_buffer.clone()).await;
}

/// Update state machine.
fn tick(video: Arc<Video>) {
    match video.state {
        MachineState::Load => load_media_segment(video),
        MachineState::Switch => spawn_local(switch_quality(video)),
        MachineState::Flush => flush_buffer(video),
        MachineState::Timeout => on_timeout(video),
        /* 5 => check_status(video),
        6 => check_abr(video),
        _ => {} */
    }
}

/// Wait 1 second then update state machine.
fn on_timeout(video: Arc<Video>) {
    let window = video.window.clone();
    let hanlde = video.handle.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Timeout");

        tick(video.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);

    match window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.into_js_value().unchecked_ref(),
        1000,
    ) {
        Ok(handle) => video.handle = handle,
        Err(e) => ConsoleService::error(&format!("{:?}", e)),
    }
}

/// Recalculate download speed EMA then set quality level.
fn check_abr(video: Arc<Video>) {
    let mut level = video.level;
    let mut switch_level = false;

    if let Some(moving_average) = video.ema.recalculate_average() {
        match level {
            0 => {
                if (moving_average + 500.0) < MEDIA_LENGTH_MS {
                    level += 1;
                    switch_level = true;
                }
            }
            1 => {
                if moving_average > MEDIA_LENGTH_MS {
                    level -= 1;
                    switch_level = true;
                } else if (moving_average + 500.0) < MEDIA_LENGTH_MS {
                    level += 1;
                    switch_level = true;
                }
            }
            2 => {
                if moving_average > MEDIA_LENGTH_MS {
                    level -= 1;
                    switch_level = true;
                } else if (moving_average + 500.0) < MEDIA_LENGTH_MS {
                    level += 1;
                    switch_level = true;
                }
            }
            3 => {
                if moving_average > MEDIA_LENGTH_MS {
                    level -= 1;
                    switch_level = true;
                }
            }
            _ => {
                panic!("Quality level is too high");
            }
        }
    }

    if switch_level {
        video.level = level;
        spawn_local(switch_quality(video));
    } else {
        check_status(video);
    }
}

/// Check buffers and current time then trigger new action.
fn check_status(video: Arc<Video>) {
    let level = video.level;

    let source_buffer = match video.tracks.read() {
        Ok(tracks) => {
            if tracks.len() == 0 {
                #[cfg(debug_assertions)]
                ConsoleService::info("No Tracks");
                on_timeout(video.clone());
                return;
            }

            tracks[level].source_buffer.clone()
        }
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            on_timeout(video.clone());
            return;
        }
    };

    let time_ranges = match source_buffer.buffered() {
        Ok(tm) => tm,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::info("Not Buffered");
            on_timeout(video);
            return;
        }
    };

    let count = time_ranges.length();

    let mut buff_start = 0.0;
    let mut buff_end = 0.0;

    for i in 0..count {
        if let Ok(start) = time_ranges.start(i) {
            buff_start = start;
        }

        if let Ok(end) = time_ranges.end(i) {
            buff_end = end;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Time Range {} buffers {}s to {}s",
            i, buff_start, buff_end
        ));
    }

    let current_time = match video.media_element.as_ref() {
        Some(media_element) => media_element.current_time(),
        None => {
            #[cfg(debug_assertions)]
            ConsoleService::info("No Media Element");
            on_timeout(video);
            return;
        }
    };

    if current_time > buff_start + BACK_BUFFER_LENGTH {
        #[cfg(debug_assertions)]
        ConsoleService::info("Back Buffer Full");
        flush_buffer(video);
        return;
    }

    if buff_end >= video.metadata.duration {
        #[cfg(debug_assertions)]
        ConsoleService::info("End Of Video");
        on_timeout(video);
        return;
    }

    if current_time + FORWARD_BUFFER_LENGTH < buff_end {
        #[cfg(debug_assertions)]
        ConsoleService::info("Forward Buffer Full");
        on_timeout(video);
        return;
    }

    load_media_segment(video);
}

/// Get CID from timecode then fetch video data from ipfs
fn load_media_segment(video: Arc<Video>) {
    let level = video.level.load(Ordering::Relaxed);

    let (quality, source_buffer) = match video.tracks.read() {
        Ok(tracks) => (
            tracks[level].quality.clone(),
            tracks[level].source_buffer.clone(),
        ),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let mut buff_end = 0.0;

    if let Ok(time_ranges) = source_buffer.buffered() {
        if let Ok(end) = time_ranges.end(0) {
            buff_end = end;
        }
    }

    //if buffer is empty load at current time
    if buff_end <= 0.0 {
        let current_time = match video.media_element.as_ref() {
            Some(media_element) => media_element.current_time(),
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::info("No Media Element");
                return;
            }
        };

        if current_time > MEDIA_LENGTH {
            buff_end = current_time - MEDIA_LENGTH;
        }
    }

    let (hours, minutes, seconds) = seconds_to_timecode(buff_end);

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Loading Media Segment at timecode {}:{}:{}",
        hours, minutes, seconds
    ));

    let path = format!(
        "{}/time/hour/{}/minute/{}/second/{}/video/quality/{}",
        video.metadata.video.link.to_string(),
        hours,
        minutes,
        seconds,
        quality
    );

    let future = cat_and_buffer(path, source_buffer);

    video.ema.start_timer();
    video.state.store(6, Ordering::Relaxed);

    spawn_local(future);
}

/// Switch source buffer codec then load initialization segment.
async fn switch_quality(video: Arc<Video>) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Switching Quality");

    let level = video.level.load(Ordering::Relaxed);

    let (quality, codec, source_buffer) = match video.tracks.read() {
        Ok(tracks) => (
            tracks[level].quality.clone(),
            tracks[level].codec.clone(),
            tracks[level].source_buffer.clone(),
        ),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    if let Err(e) = source_buffer.change_type(&codec) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Level {} Quality {} Codec {} Buffer Mode {:#?}",
        level,
        quality,
        codec,
        source_buffer.mode()
    ));

    let path = format!(
        "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
        video.metadata.video.link.to_string(),
        level
    );

    cat_and_buffer(path, source_buffer.clone()).await;

    //state load segment
    video.state.store(1, Ordering::Relaxed);
}

/// Flush everything or just back buffer.
fn flush_buffer(video: Arc<Video>) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Flushing Buffer");

    let level = video.level.load(Ordering::Relaxed);

    let source_buffer = match video.tracks.read() {
        Ok(tracks) => tracks[level].source_buffer.clone(),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let time_ranges = match source_buffer.buffered() {
        Ok(tm) => tm,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::info("Not Buffered");
            return;
        }
    };

    let count = time_ranges.length();

    let mut buff_start = 0.0;
    let mut buff_end = 0.0;

    for i in 0..count {
        if let Ok(start) = time_ranges.start(i) {
            buff_start = start;
        }

        if let Ok(end) = time_ranges.end(i) {
            buff_end = end;
        }
    }

    let current_time = match video.media_element.as_ref() {
        Some(media_element) => media_element.current_time(),
        None => {
            #[cfg(debug_assertions)]
            ConsoleService::info("No Media Element");
            return;
        }
    };

    let back_buffer_start = current_time - BACK_BUFFER_LENGTH;

    //full flush except if back buffer flush is possible
    if buff_start < back_buffer_start {
        buff_end = back_buffer_start
    }

    if let Err(e) = source_buffer.remove(buff_start, buff_end) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }

    //state load segment
    video.state.store(1, Ordering::Relaxed);
}

/// Callback when source buffer is done updating.
fn on_source_buffer_update_end(video: Arc<Video>, source_buffer: &SourceBuffer) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Update End");

        tick(video.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
}

/// Callback when video element has seeked.
fn on_video_seeking(video: Arc<Video>, media_element: &HtmlMediaElement) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Seeking");

        video.state.store(3, Ordering::Relaxed);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_element.set_onseeking(Some(callback.into_js_value().unchecked_ref()));
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
 */
