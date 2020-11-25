use crate::bindings;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use web_sys::{MediaSource, MediaSourceReadyState, SourceBuffer, Url};

use yew::services::ConsoleService;

pub struct VODManager {
    media_source: MediaSource,

    source_buffer: Option<SourceBuffer>,

    pub url: String,

    test_video_seg_one: Vec<u8>,

    test_video_seg_two: Vec<u8>,
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

        //load video from ipfs
        let test_video_seg_one = bindings::ipfs_cat(
            "bafyreibcxaz3iyotaeds6vzs7xdwlpvzp3w7gy6p37i3m46iglo7nwjoou/time/hour/0/minute/0/second/0/video/quality/720p30",
        ).to_vec();

        let test_video_seg_two = bindings::ipfs_cat(
            "bafyreibcxaz3iyotaeds6vzs7xdwlpvzp3w7gy6p37i3m46iglo7nwjoou/time/hour/0/minute/0/second/4/video/quality/720p30",
        ).to_vec();

        Self {
            media_source,
            source_buffer: None,
            url,
            test_video_seg_one,
            test_video_seg_two,
        }
    }

    /* pub fn register_callbacks(&mut self) {
        let callback = Closure::wrap(Box::new(|media_source: &MediaSource| {
            #[cfg(debug_assertions)]
            ConsoleService::info(&format!("Ready State => {:?}", media_source.ready_state()));
        }) as Box<dyn Fn(&MediaSource)>);

        self.media_source
            .set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));
    } */

    pub fn add_source_buffer(&mut self) {
        let ready_state = self.media_source.ready_state();

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Ready State => {:?}", ready_state));

        if ready_state != MediaSourceReadyState::Open {
            return;
        }

        let mime_type = r#"video/mp4; codecs="avc1.42c01f, mp4a.40.2""#;

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("MIME Type {:?}", mime_type));

        if !MediaSource::is_type_supported(mime_type) {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Unsupported");

            return;
        }

        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaSource.html#method.add_source_buffer
        //https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter
        let source_buffer = match self.media_source.add_source_buffer(mime_type) {
            Ok(sb) => sb,
            Err(e) => {
                #[cfg(debug_assertions)]
                ConsoleService::warn(&format!("{:?}", e));
                return;
            }
        };

        let callback = Closure::wrap(Box::new(on_update_end) as Box<dyn Fn()>);

        source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_update_start) as Box<dyn Fn()>);

        source_buffer.set_onupdatestart(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_error) as Box<dyn Fn()>);

        source_buffer.set_onerror(Some(callback.into_js_value().unchecked_ref()));

        self.source_buffer = Some(source_buffer);
    }

    pub fn load_test_video(&mut self) {
        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.SourceBuffer.html#method.append_buffer_with_u8_array
        if let Err(e) = self
            .source_buffer
            .as_ref()
            .unwrap()
            .append_buffer_with_u8_array(&mut self.test_video_seg_one)
        {
            #[cfg(debug_assertions)]
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }

        //TODO wait until on_update_end before adding more to tthe buffer

        if let Err(e) = self
            .source_buffer
            .as_ref()
            .unwrap()
            .append_buffer_with_u8_array(&mut self.test_video_seg_two)
        {
            #[cfg(debug_assertions)]
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    }
}

fn on_source_close() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceclose");
}

fn on_source_open() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceopen");
}

fn on_source_ended() {
    #[cfg(debug_assertions)]
    ConsoleService::info("onsourceended");
}

fn on_update_end() {
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
