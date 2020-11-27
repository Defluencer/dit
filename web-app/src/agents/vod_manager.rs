use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::bindings;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use web_sys::{MediaSource, MediaSourceReadyState, /* SourceBuffer, */ Url};

use yew::services::ConsoleService;

const MIME_TYPE: &str = r#"video/mp4; codecs="avc1.42c01f, mp4a.40.2""#;

pub struct VODManager {
    media_source: MediaSource,

    pub url: String,

    count: Arc<AtomicUsize>,
}

impl VODManager {
    pub fn new() -> Self {
        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaSource.html#method.new
        let media_source = MediaSource::new().unwrap();

        let callback = Closure::wrap(Box::new(on_source_open) as Box<dyn Fn()>);

        media_source.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_source_ended) as Box<dyn Fn()>);

        media_source.set_onsourceended(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_source_close) as Box<dyn Fn()>);

        media_source.set_onsourceclosed(Some(callback.into_js_value().unchecked_ref()));

        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Url.html#method.create_object_url_with_source
        let url = Url::create_object_url_with_source(&media_source).unwrap();

        let count = Arc::new(AtomicUsize::new(0));

        Self {
            media_source,
            url,
            count,
        }
    }

    pub fn add_source_buffer(&mut self) {
        let ready_state = self.media_source.ready_state();

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Ready State => {:?}", ready_state));

        if ready_state != MediaSourceReadyState::Open {
            return;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("MIME Type {:?}", MIME_TYPE));

        if !MediaSource::is_type_supported(MIME_TYPE) {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Unsupported");

            return;
        }

        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaSource.html#method.add_source_buffer
        //https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter
        let source_buffer = match self.media_source.add_source_buffer(MIME_TYPE) {
            Ok(sb) => sb,
            Err(e) => {
                #[cfg(debug_assertions)]
                ConsoleService::warn(&format!("{:?}", e));
                return;
            }
        };

        let init_segment = bindings::ipfs_cat("bafyreic6hsipoya2rpn3eankfplts6yvxevuztakn2uof4flnbt2ipwlue/time/hour/0/minute/0/second/0/video/init/720p30");

        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.SourceBuffer.html#method.append_buffer_with_array_buffer_view
        if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(&init_segment) {
            //This is bugged doesn't work. Overload resolution failed.
            #[cfg(debug_assertions)]
            ConsoleService::warn(&format!("{:?}", e));

            return;
        }

        let source_buffer = Arc::new(source_buffer);

        let sb = source_buffer.clone();

        let atomic_count = self.count.clone();

        let callback = Closure::wrap(Box::new(move || {
            #[cfg(debug_assertions)]
            ConsoleService::info("onupdateend");

            let count = atomic_count.load(Ordering::SeqCst);

            if count > 20 {
                return;
            }

            let path = format!("bafyreic6hsipoya2rpn3eankfplts6yvxevuztakn2uof4flnbt2ipwlue/time/hour/0/minute/0/second/{}/video/quality/720p30", count);

            let video_segment = bindings::ipfs_cat(&path);

            atomic_count.fetch_add(4, Ordering::SeqCst);

            if let Err(e) = sb.append_buffer_with_array_buffer_view(&video_segment) {
                #[cfg(debug_assertions)]
                ConsoleService::warn(&format!("{:?}", e));
            }
        }) as Box<dyn Fn()>);
        source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_update_start) as Box<dyn Fn()>);
        source_buffer.set_onupdatestart(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_error) as Box<dyn Fn()>);
        source_buffer.set_onerror(Some(callback.into_js_value().unchecked_ref()));
    }
}

fn on_source_close() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceclose");
}

fn on_source_open() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceopen");

    //This is bugged doesn't work. media_source.ready_state() != Open at this point
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
}
