//use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::bindings;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, /* SourceBuffer, */ Url};

use yew::services::ConsoleService;

use js_sys::Uint8Array;

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

    let media_source = Arc::new(MediaSource::new().unwrap());
    let media_source_clone = media_source.clone();

    let url = Url::create_object_url_with_source(&media_source).unwrap();

    video.set_src(&url);

    let callback = Closure::wrap(Box::new(move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("onsourceopen");

        let ready_state = media_source.ready_state();

        if ready_state != MediaSourceReadyState::Open {
            ConsoleService::error(&format!("Ready State => {:?}", ready_state));
            return;
        }

        let source_buffer = match media_source.add_source_buffer(MIME_TYPE) {
            Ok(sb) => sb,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let source_buffer = Arc::new(source_buffer);
        let source_buffer_clone = source_buffer.clone();

        let path = format!(
            "{}/time/hour/0/minute/0/second/0/video/init/720p30",
            TEST_CID
        );

        let init_segment: Uint8Array = bindings::ipfs_cat(&path);

        if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(&init_segment) {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }

        let callback = Closure::wrap(Box::new(move || {
            #[cfg(debug_assertions)]
            ConsoleService::info("onupdateend");

            let path = format!(
                "{}/time/hour/0/minute/0/second/0/video/quality/720p30",
                TEST_CID
            );

            let video_segment: Uint8Array = bindings::ipfs_cat(&path);

            if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(&video_segment) {
                ConsoleService::error(&format!("{:?}", e));
            }
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
}

/* fn on_source_close() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceclose");
}

fn on_source_open() {
    //This is bugged and doesn't work. media_source.ready_state() != Open at this point

    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceopen");
}

fn on_source_ended() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceended");
}

fn _on_update_end() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onupdateend");
}

fn on_update_start() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onupdatestart");
}

fn on_error() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onerror");
} */
