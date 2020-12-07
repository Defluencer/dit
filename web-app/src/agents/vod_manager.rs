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

type Tracks = Arc<RwLock<Vec<Track>>>;

struct Track {
    level: usize,
    quality: String,
    codec: String,
    source_buffer: SourceBuffer,
}

pub fn load_video(video_cid: String, duration: f64) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let video_element: HtmlMediaElement = document
        .get_element_by_id("video")
        .unwrap()
        .dyn_into()
        .unwrap();

    let media_source = MediaSource::new().unwrap();

    let url = Url::create_object_url_with_source(&media_source).unwrap();

    video_element.set_src(&url);

    let tracks = Arc::new(RwLock::new(Vec::with_capacity(4)));

    let current_level = Arc::new(AtomicUsize::new(0));

    /* video_on_seeking(
        video_cid.clone(),
        second.clone(),
        minute.clone(),
        hour.clone(),
        video.clone(),
        buffers.clone(),
        level.clone(),
    ); */

    on_media_source_open(video_cid, duration, media_source, tracks);
}

fn on_media_source_open(
    video_cid: String,
    duration: f64,
    media_source: MediaSource,
    tracks: Tracks,
) {
    let media_source_clone = media_source.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("sourceopen");

        media_source.set_onsourceopen(None);

        media_source.set_duration(duration);

        let future = add_source_buffers(
            video_cid.clone(),
            duration,
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
    duration: f64,
    media_source: MediaSource,
    tracks: Tracks,
) {
    let codecs_path = "/time/hour/0/minute/0/second/0/video/setup/codec";
    let qualities_path = "/time/hour/0/minute/0/second/0/video/setup/quality";

    let codecs_result = match bindings::ipfs_dag_get(&video_cid, codecs_path).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let qualities_result = match bindings::ipfs_dag_get(&video_cid, qualities_path).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let codecs: Vec<String> = from_value(codecs_result).expect("Can't deserialize codecs");
    let qualities: Vec<String> = from_value(qualities_result).expect("Can't deserialize qualities");

    let mut new_tracks = Vec::with_capacity(4);

    for (level, (codec, quality)) in codecs.into_iter().zip(qualities.into_iter()).enumerate() {
        if !MediaSource::is_type_supported(&codec) {
            ConsoleService::warn(&format!("MIME Type {:?} unsupported", &codec));
            continue;
        }

        let source_buffer = media_source
            .add_source_buffer(&codec)
            .expect("Can't add source buffer");

        if level == 0 {
            // Only lowest quality start buffering.
            on_source_buffer_update_end(
                video_cid.clone(),
                duration,
                quality.clone(),
                media_source.clone(),
                source_buffer.clone(),
            );
        }

        let path = format!(
            "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
            &video_cid, level
        );

        cat_and_buffer(path, source_buffer.clone()).await;

        new_tracks.push(Track {
            level,
            codec,
            quality,
            source_buffer,
        });
    }

    if let Ok(mut tracks) = tracks.write() {
        *tracks = new_tracks;
    }
}

fn on_source_buffer_update_end(
    video_cid: String,
    duration: f64,
    quality: String,
    media_source: MediaSource,
    source_buffer: SourceBuffer,
) {
    let source_buffer_clone = source_buffer.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("updateend");

        //TODO update adaptative bit rate

        append_next_segment(
            &video_cid,
            duration,
            &quality,
            media_source.clone(),
            source_buffer.clone(),
        );

        //TODO if has buffered enough then source_buffer.set_onupdateend(None);
        // and set timer for periodic buffer health checks
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    source_buffer_clone.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
}

fn video_on_seeking(
    video_cid: String,
    duration: f64,
    video: HtmlMediaElement,
    media_source: MediaSource,
    current_level: Arc<AtomicUsize>,
    tracks: Tracks,
) {
    let video_clone = video.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("seeking");

        let level = current_level.load(Ordering::SeqCst);
        let tracks = tracks.read().expect("Lock Poisoned");

        let source_buffer = tracks[level].source_buffer.clone();
        let quality = &tracks[level].quality;

        if let Err(e) = source_buffer.abort() {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }

        let (hours, minutes, seconds) = match source_buffer.buffered() {
            Ok(time_ranges) => match time_ranges.end(0) {
                Ok(end) => {
                    let seek_end_buff = video.current_time() + 30.0;

                    if end >= seek_end_buff {
                        return;
                    } else {
                        seconds_to_timecode(end)
                    }
                }
                Err(_) => seconds_to_timecode(video.current_time()),
            },
            Err(_) => seconds_to_timecode(video.current_time()),
        };

        //TODO buffer new segment
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    video_clone.set_onseeking(Some(callback.into_js_value().unchecked_ref()));
}

fn append_next_segment(
    video_cid: &str,
    duration: f64,
    quality: &str,
    media_source: MediaSource,
    source_buffer: SourceBuffer,
) {
    let (hours, minutes, seconds) = match source_buffer.buffered() {
        Ok(time_ranges) => match time_ranges.end(0) {
            Ok(end) => {
                if end >= duration {
                    #[cfg(debug_assertions)]
                    ConsoleService::info("end of stream");

                    media_source.end_of_stream().unwrap();
                    return;
                } else {
                    //TODO change hard-coded media segment length
                    seconds_to_timecode(end + 4.0)
                }
            }
            Err(_) => (0, 0, 0),
        },
        Err(_) => (0, 0, 0),
    };

    let path = format!(
        "{}/time/hour/{}/minute/{}/second/{}/video/quality/{}",
        &video_cid, hours, minutes, seconds, quality
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

    source_buffer
        .append_buffer_with_array_buffer_view(segment)
        .expect("Can't append buffer");
}

async fn wait_for_buffer(source_buffer: SourceBuffer) {
    let closure = move || !source_buffer.updating();

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn() -> bool>);

    bindings::wait_until(callback.into_js_value().unchecked_ref()).await
}

fn seconds_to_timecode(seconds: f64) -> (u8, u8, u8) {
    let hours = (seconds / 3600.0) as u8;
    let rem_seconds = seconds.rem_euclid(3600.0);

    let minutes = (rem_seconds / 60.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(60.0);

    let seconds = rem_seconds as u8;

    (hours, minutes, seconds)
}
