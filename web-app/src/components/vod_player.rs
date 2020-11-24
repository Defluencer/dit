use crate::bindings;
use wasm_bindgen::closure::Closure;
//use crate::vod_manager::VODManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::ConsoleService;

use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, Url};

//use js_sys::Uint8Array;

use wasm_bindgen::JsCast;

pub struct VODPlayer {}

impl Component for VODPlayer {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <video id="video" muted=true controls=true poster="../live_like_poster.png">
                </video>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            //https://medium.com/canal-tech/how-video-streaming-works-on-the-web-an-introduction-7919739f7e1

            let window = web_sys::window().unwrap();

            let document = window.document().unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Document.html#method.get_element_by_id
            let video: HtmlMediaElement = document
                .get_element_by_id("video")
                .unwrap()
                .dyn_into()
                .unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaSource.html#method.new
            let media_source = MediaSource::new().unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Url.html#method.create_object_url_with_source
            let url = Url::create_object_url_with_source(&media_source).unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.HtmlVideoElement.html#method.set_src
            video.set_src(&url);

            let closure = Closure::wrap(Box::new(on_source_open) as Box<dyn Fn(&MediaSource)>);

            media_source.set_onsourceopen(Some(closure.into_js_value().unchecked_ref()));
        }
    }

    /* fn destroy(&mut self) {
        bindings::destroy();
    } */
}

fn on_source_open(media_source: &MediaSource) {
    if media_source.ready_state() != MediaSourceReadyState::Open {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Ready State => {:?}", media_source.ready_state()));

        //TODO find why ready state return __nonexhaustive
        return;
    }

    //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaSource.html#method.add_source_buffer
    //https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter
    let source_buffer = media_source
        .add_source_buffer(r#"video/mp4;codecs="avc1.42c01f,mp4a.40.2""#)
        .unwrap();

    //load video from ipfs
    let data = bindings::ipfs_cat(
    "bafyreibcxaz3iyotaeds6vzs7xdwlpvzp3w7gy6p37i3m46iglo7nwjoou/time/hour/0/minute/0/second/0/video/quality/1080p60",
);

    //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.SourceBuffer.html#method.append_buffer_with_u8_array
    source_buffer
        .append_buffer_with_array_buffer_view(&data)
        .unwrap();
}
