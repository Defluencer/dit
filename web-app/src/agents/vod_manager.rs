use crate::bindings;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use js_sys::Uint8Array;
use web_sys::{HtmlMediaElement, MediaSource, SourceBuffer, Url};

use yew::services::ConsoleService;

const TEST_CID: &str = "bafyreic6hsipoya2rpn3eankfplts6yvxevuztakn2uof4flnbt2ipwlue";
const MIME_TYPE: &str = r#"video/mp4; codecs="avc1.42c01f, mp4a.40.2""#;

pub fn load_video() {
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

    let media_source = Arc::new(MediaSource::new().unwrap()); //move into closure
    let media_source_clone = media_source.clone(); // used to set callback

    let url = Url::create_object_url_with_source(&media_source).unwrap();

    video.set_src(&url);

    let seconds = Arc::new(AtomicUsize::new(0));
    let minutes = Arc::new(AtomicUsize::new(0));
    let hours = Arc::new(AtomicUsize::new(0));

    // media_source sourceopen callback
    let callback = Closure::wrap(Box::new(move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("onsourceopen");

        let source_buffer = match media_source.add_source_buffer(MIME_TYPE) {
            Ok(sb) => sb,
            Err(e) => {
                ConsoleService::warn(&format!("{:?}", e));
                return;
            }
        };

        let path = format!(
            "{}/time/hour/0/minute/0/second/0/video/init/720p30",
            TEST_CID
        );

        let source_buffer = Arc::new(source_buffer); // move into future
        let source_buffer_clone = source_buffer.clone(); // used to set callback

        spawn_local(cat_and_buffer(path, source_buffer));

        let seconds = seconds.clone();
        let minutes = minutes.clone();
        let hours = hours.clone();

        let source_buffer = source_buffer_clone.clone(); // move into closure

        // source_buffer updateend callback
        let callback = Closure::wrap(Box::new(move || {
            #[cfg(debug_assertions)]
            ConsoleService::info("onupdateend");

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

            let path = format!(
                "{}/time/hour/{}/minute/{}/second/{}/video/quality/720p30",
                TEST_CID, current_hours, current_minutes, current_seconds,
            );

            spawn_local(cat_and_buffer(path, source_buffer.clone()));
        }) as Box<dyn Fn()>);

        source_buffer_clone.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
    }) as Box<dyn Fn()>);

    media_source_clone.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));

    /* let callback = Closure::wrap(Box::new(on_source_ended) as Box<dyn Fn()>);
    media_source.set_onsourceended(Some(callback.into_js_value().unchecked_ref())); */

    /* let callback = Closure::wrap(Box::new(on_source_close) as Box<dyn Fn()>);
    media_source.set_onsourceclosed(Some(callback.into_js_value().unchecked_ref())); */

    /* let callback = Closure::wrap(Box::new(on_update_start) as Box<dyn Fn()>);
    source_buffer.set_onupdatestart(Some(callback.into_js_value().unchecked_ref())); */

    /* let callback = Closure::wrap(Box::new(on_error) as Box<dyn Fn()>);
    source_buffer.set_onerror(Some(callback.into_js_value().unchecked_ref())); */

    /* let callback = Closure::wrap(Box::new(on_abort) as Box<dyn Fn()>);
    video.set_onabort(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_canplay) as Box<dyn Fn()>);
    video.set_oncanplay(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_canplaythrough) as Box<dyn Fn()>);
    video.set_oncanplaythrough(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_durationchange) as Box<dyn Fn()>);
    video.set_ondurationchange(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_emptied) as Box<dyn Fn()>);
    video.set_onemptied(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_ended) as Box<dyn Fn()>);
    video.set_onended(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_ended) as Box<dyn Fn()>);
    video.set_onended(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_error) as Box<dyn Fn()>);
    video.set_onerror(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_loadeddata) as Box<dyn Fn()>);
    video.set_onloadeddata(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_loadedmetadata) as Box<dyn Fn()>);
    video.set_onloadedmetadata(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_loadstart) as Box<dyn Fn()>);
    video.set_onloadstart(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_pause) as Box<dyn Fn()>);
    video.set_onpause(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_play) as Box<dyn Fn()>);
    video.set_onplay(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_playing) as Box<dyn Fn()>);
    video.set_onplaying(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_progress) as Box<dyn Fn()>);
    video.set_onprogress(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_ratechange) as Box<dyn Fn()>);
    video.set_onratechange(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_seeked) as Box<dyn Fn()>);
    video.set_onseeked(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_seeking) as Box<dyn Fn()>);
    video.set_onseeking(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_stalled) as Box<dyn Fn()>);
    video.set_onstalled(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_suspend) as Box<dyn Fn()>);
    video.set_onsuspend(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_timeupdate) as Box<dyn Fn()>);
    video.set_ontimeupdate(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_volumechange) as Box<dyn Fn()>);
    video.set_onvolumechange(Some(callback.into_js_value().unchecked_ref()));

    let callback = Closure::wrap(Box::new(on_waiting) as Box<dyn Fn()>);
    video.set_onwaiting(Some(callback.into_js_value().unchecked_ref())); */
}

async fn cat_and_buffer(path: String, source_buffer: Arc<SourceBuffer>) {
    let init_segment = match bindings::ipfs_cat(&path).await {
        Ok(vs) => vs,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let init_segment: &Uint8Array = init_segment.unchecked_ref();

    if source_buffer.updating() {
        ConsoleService::warn("Buffer still updating abort append buffer");
        return;
    }

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(init_segment) {
        ConsoleService::warn(&format!("{:?}", e));
        return;
    }
}

/* fn on_source_close() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceclose");
}

fn on_source_ended() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceended");
}

fn on_update_start() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onupdatestart");
}

fn on_error() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onerror");
} */

/* fn on_abort() {
    #[cfg(debug_assertions)]
    ConsoleService::info("abort");
} */

/* fn on_canplay() {
    #[cfg(debug_assertions)]
    ConsoleService::info("canplay");
} */

/* fn on_canplaythrough() {
    #[cfg(debug_assertions)]
    ConsoleService::info("canplaythrough");
} */

/* fn on_durationchange() {
    #[cfg(debug_assertions)]
    ConsoleService::info("durationchange");
} */

/* fn on_emptied() {
    #[cfg(debug_assertions)]
    ConsoleService::info("emptied");
} */

/* fn on_ended() {
    #[cfg(debug_assertions)]
    ConsoleService::info("ended");
} */

/* fn on_error() {
    #[cfg(debug_assertions)]
    ConsoleService::info("error");
} */

/* fn on_loadeddata() {
    #[cfg(debug_assertions)]
    ConsoleService::info("loadeddata");
} */

/* fn on_loadedmetadata() {
    #[cfg(debug_assertions)]
    ConsoleService::info("loadedmetadata");
} */

/* fn on_loadstart() {
    #[cfg(debug_assertions)]
    ConsoleService::info("loadstart");
} */

/* fn on_pause() {
    #[cfg(debug_assertions)]
    ConsoleService::info("pause");
} */

/* fn on_play() {
    #[cfg(debug_assertions)]
    ConsoleService::info("play");
} */

/* fn on_playing() {
    #[cfg(debug_assertions)]
    ConsoleService::info("playing");
} */

/* fn on_progress() {
    #[cfg(debug_assertions)]
    ConsoleService::info("progress");
} */

/* fn on_ratechange() {
    #[cfg(debug_assertions)]
    ConsoleService::info("ratechange");
} */

/* fn on_seeked() {
    #[cfg(debug_assertions)]
    ConsoleService::info("seeked");
} */

/* fn on_seeking() {
    #[cfg(debug_assertions)]
    ConsoleService::info("seeking");
} */

/* fn on_stalled() {
    #[cfg(debug_assertions)]
    ConsoleService::info("stalled");
} */

/* fn on_suspend() {
    #[cfg(debug_assertions)]
    ConsoleService::info("suspend");
} */

/* fn on_timeupdate() {
    #[cfg(debug_assertions)]
    ConsoleService::info("timeupdate");
} */

/* fn on_volumechange() {
    #[cfg(debug_assertions)]
    ConsoleService::info("volumechange");
} */

/* fn on_waiting() {
    #[cfg(debug_assertions)]
    ConsoleService::info("waiting");
} */
