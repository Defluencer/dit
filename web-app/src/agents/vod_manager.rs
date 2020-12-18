use crate::bindings;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use serde_wasm_bindgen::from_value;

use js_sys::Uint8Array;
use web_sys::{HtmlMediaElement, MediaSource, SourceBuffer, Url};

use yew::services::ConsoleService;

//const BUFFER_LENGTH: f64 = 30.0;
//const MEDIA_LENGTH: f64 = 4.0;

const INIT_LEVEL: usize = 0;
const CODEC_PATH: &str = "/time/hour/0/minute/0/second/0/video/setup/codec";
const QUALITY_PATH: &str = "/time/hour/0/minute/0/second/0/video/setup/quality";

type Tracks = Arc<RwLock<Vec<Track>>>;

struct Track {
    //level: usize,
    quality: String,
    //codec: String,
    source_buffer: SourceBuffer,
}

pub fn load_video(video_cid: String, duration: f64) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let video_element: HtmlMediaElement = document
        .get_element_by_id("video")
        .unwrap()
        .unchecked_into();

    let media_source = MediaSource::new().unwrap();

    let url = Url::create_object_url_with_source(&media_source).unwrap();

    video_element.set_src(&url);

    let tracks = Arc::new(RwLock::new(Vec::with_capacity(4)));

    let current_level = Arc::new(AtomicUsize::new(INIT_LEVEL));

    /* video_on_seeking(
        video_cid.clone(),
        video_element.clone(),
        media_source.clone(),
        current_level.clone(),
        tracks.clone(),
    ); */

    on_media_source_open(
        video_cid,
        duration,
        current_level,
        video_element,
        media_source,
        tracks,
    );
}

/// Called once the MediaSource is ready to accept segments.
fn on_media_source_open(
    video_cid: String,
    duration: f64,
    current_level: Arc<AtomicUsize>,
    video: HtmlMediaElement,
    media_source: MediaSource,
    tracks: Tracks,
) {
    let media_source_clone = media_source.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("on source open");

        media_source.set_onsourceopen(None);

        media_source.set_duration(duration);

        let future = add_source_buffers(
            video_cid.clone(),
            current_level.clone(),
            video.clone(),
            media_source.clone(),
            tracks.clone(),
        );

        spawn_local(future);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_source_clone.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));
}

async fn add_source_buffers(
    video_cid: String,
    current_level: Arc<AtomicUsize>,
    video: HtmlMediaElement,
    media_source: MediaSource,
    tracks: Tracks,
) {
    let codecs = match bindings::ipfs_dag_get(&video_cid, CODEC_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let qualities = match bindings::ipfs_dag_get(&video_cid, QUALITY_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let codecs: Vec<String> = from_value(codecs).expect("Can't deserialize codecs");
    let qualities: Vec<String> = from_value(qualities).expect("Can't deserialize qualities");

    #[cfg(debug_assertions)]
    ConsoleService::info("Creating all source buffers");

    match tracks.write() {
        Ok(mut tracks) => {
            for (level, (codec, quality)) in
                codecs.into_iter().zip(qualities.into_iter()).enumerate()
            {
                if level > 0 {
                    continue;
                }

                if !MediaSource::is_type_supported(&codec) {
                    ConsoleService::error(&format!("MIME Type {:?} unsupported", &codec));
                    continue;
                }

                let source_buffer = match media_source.add_source_buffer(&codec) {
                    Ok(sb) => sb,
                    Err(e) => {
                        ConsoleService::error(&format!("{:?}", e));
                        return;
                    }
                };

                #[cfg(debug_assertions)]
                ConsoleService::info(&format!(
                    "Level {} Quality {} Codec {} Buffer Mode {:#?}",
                    level,
                    quality,
                    codec,
                    source_buffer.mode()
                ));

                let track = Track {
                    //codec,
                    quality,
                    source_buffer,
                };

                tracks.push(track);
            }
        }
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }

    // Set callback for buffering.
    on_source_buffer_update_end(
        video_cid.clone(),
        video.clone(),
        media_source.clone(),
        current_level.clone(),
        tracks.clone(),
    );

    #[cfg(debug_assertions)]
    ConsoleService::info("Loading all initialization segments");

    match tracks.read() {
        Ok(tracks) => {
            /* let path = format!(
                "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
                &video_cid, INIT_LEVEL
            );

            cat_and_buffer(path, tracks[INIT_LEVEL].source_buffer.clone()).await; */

            //The init segment loaded first determine the track that can be played right away.
            for (level, track) in tracks.iter().enumerate() {
                if level > 0 {
                    continue;
                }

                let path = format!(
                    "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
                    &video_cid, level
                );

                cat_and_buffer(path, track.source_buffer.clone()).await;
            }
        }
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }
}

/// First called after initialization segment finish loading. Also called when append/flushing buffer.
fn on_source_buffer_update_end(
    video_cid: String,
    video: HtmlMediaElement,
    media_source: MediaSource,
    current_level: Arc<AtomicUsize>,
    tracks: Tracks,
) {
    //let level = current_level.load(Ordering::SeqCst);

    let source_buffer = match tracks.read() {
        Ok(tracks) => tracks[INIT_LEVEL].source_buffer.clone(),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("on update end");

        //TODO update adaptative bit rate
        //To switch level flush everything then append init+media????

        let level = current_level.load(Ordering::SeqCst);

        let (quality, source_buffer) = match tracks.read() {
            Ok(tracks) => (
                tracks[level].quality.clone(),
                tracks[INIT_LEVEL].source_buffer.clone(),
            ),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let current_time = video.current_time();

        #[cfg(debug_assertions)]
        {
            ConsoleService::info(&format!(
                "Video:\nLevel {}\nQuality {}\nCurrent Time {}\nReady State {}\nNetwork State {}\nError {:#?}",
                level,
                quality,
                current_time,
                video.ready_state(),
                video.network_state(),
                video.error(),
            ));
        }

        let mut buff_start = 0.0;
        let mut buff_end = 0.0;

        if let Ok(time_ranges) = source_buffer.buffered() {
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
        }

        let duration = media_source.duration();

        /* if buff_end > current_time + BUFFER_LENGTH {
            #[cfg(debug_assertions)]
            ConsoleService::info("stop buffering");

            source_buffer.set_onupdateend(None);

            //TODO set timer for periodic buffer health checks

            return;
        } */

        if buff_end + 0.5 > duration {
            #[cfg(debug_assertions)]
            ConsoleService::info("video end");

            source_buffer.set_onupdateend(None);

            /* if let Err(e) = media_source.end_of_stream() {
                ConsoleService::warn(&format!("{:?}", e));
                return;
            } */

            //current_level.store(3, Ordering::SeqCst);

            let new_quality = "1080p60";

            match tracks.write() {
                Ok(mut tracks) => {
                    tracks[0].quality = new_quality.into();
                }
                Err(e) => {
                    ConsoleService::error(&format!("{:?}", e));
                    return;
                }
            }

            if let Err(e) = source_buffer.remove(0.0, f64::INFINITY) {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }

            video.set_current_time(0.0);

            let path = format!(
                "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
                &video_cid, 3
            );

            let future = cat_and_buffer(path, source_buffer.clone());

            spawn_local(future);

            on_source_buffer_update_end(
                video_cid.clone(),
                video.clone(),
                media_source.clone(),
                current_level.clone(),
                tracks.clone(),
            );

            append_media_segment(&video_cid, new_quality, 0, 0, 0, source_buffer);

            return;
        }

        /* if buff_end < current_time {
            buff_end = current_time;
        } */

        let (hours, minutes, seconds) = seconds_to_timecode(buff_end);

        append_media_segment(&video_cid, &quality, hours, minutes, seconds, source_buffer);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
}

fn _switch_level(
    video_cid: String,
    video: HtmlMediaElement,
    media_source: MediaSource,
    current_level: Arc<AtomicUsize>,
    tracks: Tracks,
) {
    current_level.store(3, Ordering::SeqCst);

    let source_buffer = match tracks.read() {
        Ok(tracks) => tracks[0].source_buffer.clone(),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    //clean up previous level buffer
    if let Err(e) = source_buffer.remove(0.0, f64::INFINITY) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }

    video.set_current_time(0.0);

    let path = format!(
        "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
        &video_cid, 3
    );

    let future = cat_and_buffer(path, source_buffer.clone());

    spawn_local(future);

    on_source_buffer_update_end(
        video_cid.clone(),
        video,
        media_source,
        current_level,
        tracks,
    );

    append_media_segment(&video_cid, "1080p60", 0, 0, 0, source_buffer);
}

/* fn video_on_seeking(
    video_cid: String,
    video: HtmlMediaElement,
    media_source: MediaSource,
    current_level: Arc<AtomicUsize>,
    tracks: Tracks,
) {
    let video_clone = video.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("on seeking");

        let current_time = video.current_time();

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Seek at {}s", current_time));

        let level = current_level.load(Ordering::SeqCst);

        let source_buffer = if let Ok(tracks) = tracks.read() {
            tracks[level].source_buffer.clone()
        } else {
            return;
        };

        let mut buff_start = 0.0;
        let mut buff_end = 0.0;

        if let Ok(time_ranges) = source_buffer.buffered() {
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
                    "Buffer time range {} = {}s to {}s",
                    i, buff_start, buff_end
                ));
            }
        }

        if buff_end > current_time {
            return;
        }

        if source_buffer.updating() {
            if let Err(e) = source_buffer.abort() {
                ConsoleService::warn(&format!("{:?}", e));
                return;
            }
        }

        on_source_buffer_update_end(
            video_cid.clone(),
            video.clone(),
            media_source.clone(),
            current_level.clone(),
            tracks.clone(),
        );

        if let Err(e) = source_buffer.remove(buff_start, buff_end) {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    video_clone.set_onseeking(Some(callback.into_js_value().unchecked_ref()));
} */

fn append_media_segment(
    video_cid: &str,
    quality: &str,
    hours: u8,
    minutes: u8,
    seconds: u8,
    source_buffer: SourceBuffer,
) {
    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Loading Media at timecode {}:{}:{}",
        hours, minutes, seconds
    ));

    let path = format!(
        "{}/time/hour/{}/minute/{}/second/{}/video/quality/{}",
        video_cid, hours, minutes, seconds, quality
    );

    let future = cat_and_buffer(path, source_buffer);

    spawn_local(future);
}

async fn cat_and_buffer(path: String, source_buffer: SourceBuffer) {
    let segment = match bindings::ipfs_cat(&path).await {
        Ok(vs) => vs,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let segment: &Uint8Array = segment.unchecked_ref();

    wait_for_buffer(source_buffer.clone()).await;

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(segment) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }
}

async fn wait_for_buffer(source_buffer: SourceBuffer) {
    let closure = move || !source_buffer.updating();

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn() -> bool>);

    bindings::wait_until(callback.into_js_value().unchecked_ref()).await
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
