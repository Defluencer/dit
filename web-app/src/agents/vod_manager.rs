use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use crate::utils::{cat_and_buffer, ipfs_dag_get, ExponentialMovingAverage, Track, Tracks};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use serde_wasm_bindgen::from_value;

use web_sys::{HtmlMediaElement, MediaSource, SourceBuffer, Url, Window};

use yew::services::ConsoleService;

const BUFFER_LENGTH: f64 = 30.0;
const MEDIA_LENGTH: f64 = 4.0;
const MEDIA_LENGTH_MS: f64 = 4000.0;

const CODEC_PATH: &str = "/time/hour/0/minute/0/second/0/video/setup/codec";
const QUALITY_PATH: &str = "/time/hour/0/minute/0/second/0/video/setup/quality";

#[derive(Clone)]
struct Video {
    cid: String,
    duration: f64,

    window: Window,
    media_element: Option<HtmlMediaElement>,
    media_source: MediaSource,

    tracks: Tracks,
    state: Arc<AtomicUsize>,
    level: Arc<AtomicUsize>,

    ema: ExponentialMovingAverage,

    handle: Arc<AtomicIsize>,
}

pub struct VideoOnDemandManager {
    video_record: Video,

    url: String,
}

impl VideoOnDemandManager {
    pub fn new(cid: String, duration: f64) -> Self {
        let window = web_sys::window().expect("Can't get window");

        let ema = ExponentialMovingAverage::new(&window);

        let media_source = MediaSource::new().expect("Can't create media source");

        let url = Url::create_object_url_with_source(&media_source)
            .expect("Can't create url from source");

        let video_record = Video {
            cid,
            duration,

            window,
            media_element: None,
            media_source,

            tracks: Arc::new(RwLock::new(Vec::with_capacity(4))),
            state: Arc::new(AtomicUsize::new(0)),
            level: Arc::new(AtomicUsize::new(0)),

            ema,

            handle: Arc::new(AtomicIsize::new(0)),
        };

        Self { video_record, url }
    }

    pub fn link_video(&mut self) {
        let document = self
            .video_record
            .window
            .document()
            .expect("Can't get document");

        let media_element: HtmlMediaElement = document
            .get_element_by_id("video")
            .expect("No element with this Id")
            .unchecked_into();

        on_media_source_open(self.video_record.clone(), &self.video_record.media_source);

        on_video_seeking(self.video_record.clone(), &media_element);

        media_element.set_src(&self.url);

        self.video_record.media_element = Some(media_element);
    }
}

impl Drop for VideoOnDemandManager {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping VideoOnDemandManager");

        let handle = self.video_record.handle.load(Ordering::Relaxed);

        if handle != 0 {
            self.video_record
                .window
                .clear_interval_with_handle(handle as i32);
        }
    }
}

/// Called once the MediaSource is ready to accept segments.
fn on_media_source_open(video_record: Video, media_source: &MediaSource) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Source Open");

        video_record.media_source.set_onsourceopen(None);

        tick(video_record.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_source.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));
}

fn tick(video_record: Video) {
    let mut current_level = video_record.level.load(Ordering::Relaxed);

    let source_buffer = match video_record.tracks.read() {
        Ok(tracks) => tracks[current_level].source_buffer.clone(),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let mut current_state = video_record.state.load(Ordering::Relaxed);

    if let Some(moving_average) = video_record.ema.recalculate_average() {
        match current_level {
            0 => {
                if (moving_average + 500.0) < MEDIA_LENGTH_MS {
                    current_state = 2;
                    current_level += 1;
                }
            }
            1 => {
                if moving_average > MEDIA_LENGTH_MS {
                    current_state = 2;
                    current_level -= 1;
                } else if (moving_average + 500.0) < MEDIA_LENGTH_MS {
                    current_state = 2;
                    current_level += 1;
                }
            }
            2 => {
                if moving_average > MEDIA_LENGTH_MS {
                    current_state = 2;
                    current_level -= 1;
                } else if (moving_average + 500.0) < MEDIA_LENGTH_MS {
                    current_state = 2;
                    current_level += 1;
                }
            }
            3 => {
                if moving_average > MEDIA_LENGTH_MS {
                    current_state = 2;
                    current_level -= 1;
                }
            }
            _ => {}
        }
    }

    if let Ok(time_ranges) = source_buffer.buffered() {
        let mut buff_start = 0.0;
        let mut buff_end = 0.0;

        let count = time_ranges.length();

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

        //TODO if seek time is not in buffer flush everything then load segment

        //flush back buffer if your not switching level
        if (buff_end - buff_start) > BUFFER_LENGTH && current_state != 2 {
            //state flush buffer
            current_state = 3;
        }

        if let Some(media_element) = video_record.media_element.as_ref() {
            let current_time = media_element.current_time();

            //stop loading segment if buffer full and not flushing
            if buff_end > current_time + BUFFER_LENGTH && current_state != 3 {
                current_state = 4;
            }

            //stop loading segment if at the end and not flushing
            if buff_end >= video_record.duration && current_state != 3 {
                current_state = 4;
            }
        }
    }

    video_record.state.store(current_state, Ordering::Relaxed);
    video_record.level.store(current_level, Ordering::Relaxed);

    match current_state {
        0 => spawn_local(add_source_buffer(video_record)),
        1 => load_media_segment(video_record),
        2 => spawn_local(switch_quality(video_record)),
        3 => flush_back_buffer(video_record),
        4 => on_timeout(video_record),
        _ => {}
    }
}

fn on_timeout(video_record: Video) {
    let window = video_record.window.clone();
    let hanlde = video_record.handle.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Timeout");

        tick(video_record.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);

    match window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.into_js_value().unchecked_ref(),
        1000,
    ) {
        Ok(handle) => hanlde.store(handle as isize, Ordering::Relaxed),
        Err(e) => ConsoleService::error(&format!("{:?}", e)),
    }
}

async fn add_source_buffer(video_record: Video) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Adding Source Buffer");

    //TODO dag get to whole node the deserialize it

    let codecs = match ipfs_dag_get(&video_record.cid, CODEC_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let qualities = match ipfs_dag_get(&video_record.cid, QUALITY_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let codecs: Vec<String> = from_value(codecs).expect("Can't deserialize codecs");
    let qualities: Vec<String> = from_value(qualities).expect("Can't deserialize qualities");

    let mut vec = Vec::with_capacity(4);

    let first_codec = codecs.first().expect("Can't get first codec");

    let source_buffer = match video_record.media_source.add_source_buffer(first_codec) {
        Ok(sb) => sb,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info("Listing Tracks");

    for (level, (codec, quality)) in codecs.into_iter().zip(qualities.into_iter()).enumerate() {
        if !MediaSource::is_type_supported(&codec) {
            ConsoleService::error(&format!("MIME Type {:?} unsupported", &codec));
            return;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Level {} Quality {} Codec {}",
            level, quality, codec
        ));

        let track = Track {
            level,
            quality,
            codec,
            source_buffer: source_buffer.clone(),
        };

        vec.push(track);
    }

    on_source_buffer_update_end(video_record.clone(), &source_buffer);

    let path = format!(
        "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
        video_record.cid, 0
    );

    cat_and_buffer(path, source_buffer.clone()).await;

    match video_record.tracks.write() {
        Ok(mut tracks) => *tracks = vec,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }

    //state load segment
    video_record.state.store(1, Ordering::Relaxed);
}

fn load_media_segment(video_record: Video) {
    let level = video_record.level.load(Ordering::Relaxed);

    let (quality, source_buffer) = match video_record.tracks.read() {
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

    let (hours, minutes, seconds) = seconds_to_timecode(buff_end);

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Loading Media Segment at timecode {}:{}:{}",
        hours, minutes, seconds
    ));

    let path = format!(
        "{}/time/hour/{}/minute/{}/second/{}/video/quality/{}",
        video_record.cid, hours, minutes, seconds, quality
    );

    let future = cat_and_buffer(path, source_buffer);

    video_record.ema.start_timer();

    spawn_local(future);
}

async fn switch_quality(video_record: Video) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Switching Quality");

    let level = video_record.level.load(Ordering::Relaxed);

    let (quality, codec, source_buffer) = match video_record.tracks.read() {
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
        video_record.cid, level
    );

    cat_and_buffer(path, source_buffer.clone()).await;

    //state load segment
    video_record.state.store(1, Ordering::Relaxed);
}

fn flush_back_buffer(video_record: Video) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Flushing Back Buffer");

    let level = video_record.level.load(Ordering::Relaxed);

    let source_buffer = match video_record.tracks.read() {
        Ok(tracks) => tracks[level].source_buffer.clone(),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let mut buff_start = 0.0;

    if let Ok(time_ranges) = source_buffer.buffered() {
        if let Ok(start) = time_ranges.start(0) {
            buff_start = start;
        }
    }

    if let Err(e) = source_buffer.remove(buff_start, buff_start + MEDIA_LENGTH) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }

    //state load segment
    video_record.state.store(1, Ordering::Relaxed);
}

fn on_source_buffer_update_end(video_record: Video, source_buffer: &SourceBuffer) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Update End");

        tick(video_record.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
}

fn on_video_seeking(video_record: Video, media_element: &HtmlMediaElement) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Seeking");

        tick(video_record.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_element.set_onseeking(Some(callback.into_js_value().unchecked_ref()));
}

fn seconds_to_timecode(seconds: f64) -> (u8, u8, u8) {
    let rem_seconds = seconds.round();

    let hours = (rem_seconds / 3600.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(3600.0);

    let minutes = (rem_seconds / 60.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(60.0);

    let seconds = rem_seconds as u8;

    (hours, minutes, seconds)
}
