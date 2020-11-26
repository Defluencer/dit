use crate::bindings;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use web_sys::{MediaSource, MediaSourceReadyState, SourceBuffer, Url};

use yew::services::ConsoleService;

const MIME_TYPE: &str = r#"video/mp4;codecs="avc1.42c01f,mp4a.40.2""#;

pub struct VODManager {
    media_source: MediaSource,

    source_buffer: Option<SourceBuffer>,

    pub url: String,

    count: usize,
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

        Self {
            media_source,
            source_buffer: None,
            url,
            count: 0,
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

        let callback = Closure::wrap(Box::new(on_update_end) as Box<dyn Fn()>);
        source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_update_start) as Box<dyn Fn()>);
        source_buffer.set_onupdatestart(Some(callback.into_js_value().unchecked_ref()));

        let callback = Closure::wrap(Box::new(on_error) as Box<dyn Fn()>);
        source_buffer.set_onerror(Some(callback.into_js_value().unchecked_ref()));

        self.source_buffer = Some(source_buffer);
    }

    pub fn load_init_segment(&mut self) {
        //load video from ipfs
        let mut test_video_init = bindings::ipfs_cat(
            "bafyreic6hsipoya2rpn3eankfplts6yvxevuztakn2uof4flnbt2ipwlue/time/hour/0/minute/0/second/0/video/init/720p30",
        ).to_vec();

        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.SourceBuffer.html#method.append_buffer_with_u8_array
        if let Err(e) = self
            .source_buffer
            .as_ref()
            .unwrap()
            .append_buffer_with_u8_array(&mut test_video_init)
        {
            #[cfg(debug_assertions)]
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    }

    pub fn load_test_video(&mut self) {
        let path = format!("bafyreic6hsipoya2rpn3eankfplts6yvxevuztakn2uof4flnbt2ipwlue/time/hour/0/minute/0/second/{}/video/quality/720p30", self.count);

        self.count += 4;

        let mut test_video = bindings::ipfs_cat(&path).to_vec();

        //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.SourceBuffer.html#method.append_buffer_with_u8_array
        if let Err(e) = self
            .source_buffer
            .as_ref()
            .unwrap()
            .append_buffer_with_u8_array(&mut test_video)
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
