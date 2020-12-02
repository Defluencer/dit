use crate::bindings;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use js_sys::Uint8Array;
use web_sys::{HtmlMediaElement, MediaSource, SourceBuffer, Url};

use yew::services::ConsoleService;

const MIME_TYPE: &str = r#"video/mp4; codecs="avc1.42c01f, mp4a.40.2""#;

pub fn load_video(video_cid: String) {
    if !MediaSource::is_type_supported(MIME_TYPE) {
        ConsoleService::warn(&format!("MIME Type {:?} unsupported", MIME_TYPE));
        return;
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let video: HtmlMediaElement = document
        .get_element_by_id("video")
        .unwrap()
        .dyn_into()
        .unwrap();

    let media_source = MediaSource::new().unwrap();

    let url = Url::create_object_url_with_source(&media_source).unwrap();

    video.set_src(&url);

    let seconds = Arc::new(AtomicUsize::new(0));
    let minutes = Arc::new(AtomicUsize::new(0));
    let hours = Arc::new(AtomicUsize::new(0));

    media_source_on_source_open(
        video_cid.clone(),
        seconds.clone(),
        minutes.clone(),
        hours.clone(),
        media_source.clone(),
    );

    video_on_progress(
        video_cid.clone(),
        seconds.clone(),
        minutes.clone(),
        hours.clone(),
        video.clone(),
        media_source.clone(),
    );

    video_on_seeking(video_cid, seconds, minutes, hours, video, media_source);
}

fn media_source_on_source_open(
    video_cid: String,
    seconds: Arc<AtomicUsize>,
    minutes: Arc<AtomicUsize>,
    hours: Arc<AtomicUsize>,
    media_source: MediaSource,
) {
    let media_source_clone = media_source.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("sourceopen");

        let source_buffer = match media_source.add_source_buffer(MIME_TYPE) {
            Ok(sb) => sb,
            Err(e) => {
                ConsoleService::warn(&format!("{:?}", e));
                return;
            }
        };

        source_buffer_on_update_end(
            video_cid.clone(),
            seconds.clone(),
            minutes.clone(),
            hours.clone(),
            source_buffer.clone(),
        );

        let path = &format!(
            "{}/time/hour/0/minute/0/second/0/video/init/720p30",
            &video_cid
        );

        let future = cat_and_buffer(path, source_buffer);

        spawn_local(future);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_source_clone.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));
}

fn source_buffer_on_update_end(
    video_cid: String,
    seconds: Arc<AtomicUsize>,
    minutes: Arc<AtomicUsize>,
    hours: Arc<AtomicUsize>,
    source_buffer: SourceBuffer,
) {
    let source_buffer_clone = source_buffer.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("updateend");

        source_buffer.set_onupdateend(None);

        append_next_segment(
            &video_cid,
            seconds.clone(),
            minutes.clone(),
            hours.clone(),
            source_buffer.clone(),
        );
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    source_buffer_clone.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
}

fn video_on_progress(
    video_cid: String,
    seconds: Arc<AtomicUsize>,
    minutes: Arc<AtomicUsize>,
    hours: Arc<AtomicUsize>,
    video: HtmlMediaElement,
    media_source: MediaSource,
) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("progress");

        let source_buffer = media_source.source_buffers().get(0).unwrap();

        append_next_segment(
            &video_cid,
            seconds.clone(),
            minutes.clone(),
            hours.clone(),
            source_buffer,
        );
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    video.set_onprogress(Some(callback.into_js_value().unchecked_ref()));
}

fn video_on_seeking(
    video_cid: String,
    seconds: Arc<AtomicUsize>,
    minutes: Arc<AtomicUsize>,
    hours: Arc<AtomicUsize>,
    video: HtmlMediaElement,
    media_source: MediaSource,
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
}

fn append_next_segment(
    video_cid: &str,
    seconds: Arc<AtomicUsize>,
    minutes: Arc<AtomicUsize>,
    hours: Arc<AtomicUsize>,
    source_buffer: SourceBuffer,
) {
    let current_seconds = seconds.fetch_add(4, Ordering::SeqCst); //returns previous value

    let current_minutes = if current_seconds >= 60 {
        seconds.store(0, Ordering::SeqCst);

        minutes.fetch_add(1, Ordering::SeqCst) //returns previous value
    } else {
        minutes.load(Ordering::SeqCst)
    };

    let current_hours = if current_minutes >= 60 {
        minutes.store(0, Ordering::SeqCst);

        hours.fetch_add(1, Ordering::SeqCst) //returns previous value
    } else {
        hours.load(Ordering::SeqCst)
    };

    let path = &format!(
        "{}/time/hour/{}/minute/{}/second/{}/video/quality/720p30",
        &video_cid, current_hours, current_minutes, current_seconds,
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

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(segment) {
        ConsoleService::warn(&format!("{:?}", e));
        return;
    }
}

async fn wait_for_buffer(source_buffer: SourceBuffer) {
    let closure = move || !source_buffer.updating();

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn() -> bool>);

    bindings::wait_until(callback.into_js_value().unchecked_ref()).await
}
