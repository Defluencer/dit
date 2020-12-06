use crate::bindings;

use std::sync::atomic::AtomicU8;
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

    let _level = Arc::new(AtomicU8::new(0));

    /* video_on_seeking(
        video_cid.clone(),
        second.clone(),
        minute.clone(),
        hour.clone(),
        video.clone(),
        buffers.clone(),
        level.clone(),
    ); */

    on_media_source_add_buffer(
        video_cid.clone(),
        duration,
        media_source.clone(),
        tracks.clone(),
    );

    on_media_source_open(
        video_cid.clone(),
        duration,
        media_source.clone(),
        tracks.clone(),
    );
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

        let future = add_source_buffers(video_cid.clone(), media_source.clone(), tracks.clone());

        spawn_local(future);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_source_clone.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));
}

async fn add_source_buffers(video_cid: String, media_source: MediaSource, tracks: Tracks) {
    let codecs_path = "/time/hour/0/minute/0/second/0/video/setup/codec";
    let qualities_path = "/time/hour/0/minute/0/second/0/video/setup/quality";

    //TODO dag get setup node instead

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

    let dag_codecs: Vec<String> = from_value(codecs_result).expect("Can't deserialize codecs");
    let dag_qualities: Vec<String> =
        from_value(qualities_result).expect("Can't deserialize qualities");

    let mut new_tracks = Vec::with_capacity(4);

    for (codec, quality) in dag_codecs.into_iter().zip(dag_qualities.into_iter()) {
        if !MediaSource::is_type_supported(&codec) {
            ConsoleService::warn(&format!("MIME Type {:?} unsupported", &codec));
            continue;
        }

        let source_buffer = media_source
            .add_source_buffer(&codec)
            .expect("Can't add source buffer");

        new_tracks.push(Track {
            codec,
            quality,
            source_buffer,
        });
    }

    if let Ok(mut tracks) = tracks.write() {
        *tracks = new_tracks;
    }
}

fn on_media_source_add_buffer(
    video_cid: String,
    duration: f64,
    media_source: MediaSource,
    tracks: Tracks,
) {
    let buffer_list = media_source.source_buffers();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("addsourcebuffer");

        let list = media_source.source_buffers();

        let count = list.length();

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Buffers Count {}", count));

        let list_index = count - 1;

        // Assume last buffer is newest.
        let source_buffer = list.get(list_index).expect("Less than one buffer");

        // Search Tracks then append init segment to buffer.
        if let Ok(tracks) = tracks.read() {
            // TODO Remove search if buffer list index always equal track index.
            for (i, track) in tracks.iter().enumerate() {
                if track.source_buffer == source_buffer {
                    #[cfg(debug_assertions)]
                    ConsoleService::info(&format!("List index {}, Track index {}", list_index, i));

                    if i == 0 {
                        // Only lowest quality start buffering.
                        on_source_buffer_update_end(
                            video_cid.clone(),
                            duration,
                            track.quality.clone(),
                            media_source.clone(),
                            source_buffer.clone(),
                        );
                    }

                    let path = &format!(
                        "{}/time/hour/0/minute/0/second/0/video/setup/initseg/{}",
                        &video_cid, i
                    );

                    let future = cat_and_buffer(path, source_buffer);

                    spawn_local(future);

                    break;
                }
            }
        }
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    buffer_list.set_onaddsourcebuffer(Some(callback.into_js_value().unchecked_ref()));
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

/* fn video_on_seeking(
    video_cid: String,
    second: Arc<AtomicU8>,
    minute: Arc<AtomicU8>,
    hour: Arc<AtomicU8>,
    video: HtmlMediaElement,
    buffers: Buffers,
    level: Arc<AtomicU8>,
) {
    let video_clone = video.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("seeking");

        let source_buffer = media_source.source_buffers().get(0).unwrap();

        if let Err(e) = source_buffer.abort() {
            ConsoleService::warn(&format!("{:?}", e));
        }

        let current_time = video_clone.current_time();

        let hour = current_time as usize / 3600;
        hours.store(hour, Ordering::SeqCst);

        let rem_minutes = current_time.rem_euclid(3600.0);

        let minute = rem_minutes as usize / 60;
        minutes.store(minute, Ordering::SeqCst);

        let rem_seconds = rem_minutes.rem_euclid(60.0);

        let second = rem_seconds as usize;
        seconds.store(second, Ordering::SeqCst);

        append_next_segment(
            &video_cid,
            seconds.clone(),
            minutes.clone(),
            hours.clone(),
            source_buffer,
        );
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    video.set_onseeking(Some(callback.into_js_value().unchecked_ref()));
} */

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
                    media_source.end_of_stream();
                    return;
                } else {
                    seconds_to_timecode(end + 4.0)
                }
            }
            Err(_) => (0, 0, 0),
        },
        Err(_) => (0, 0, 0),
    };

    let path = &format!(
        "{}/time/hour/{}/minute/{}/second/{}/video/quality/{}",
        &video_cid, hours, minutes, seconds, quality
    );

    let future = cat_and_buffer(path, source_buffer);

    spawn_local(future);
}

async fn cat_and_buffer(path: &str, source_buffer: SourceBuffer) {
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
