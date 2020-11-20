//use crate::bindings;
//use crate::vod_manager::VODManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

use web_sys::{HtmlVideoElement, MediaSource, Url};

use wasm_bindgen::JsCast;

pub struct VODPlayer {
    //vod: VODManager,
}

impl Component for VODPlayer {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        //let vod = VODManager::new();

        //vod.register_callback();

        Self { /* vod */ }
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
                <video id="video" inline=true muted=true controls=true poster="/live_like_poster.png">
                </video>
                {"W.I.P."}
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            //https://medium.com/canal-tech/how-video-streaming-works-on-the-web-an-introduction-7919739f7e1

            let window = web_sys::window().unwrap();

            let document = window.document().unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Document.html#method.get_element_by_id
            let video: HtmlVideoElement = document
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

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MediaSource.html#method.add_source_buffer
            //https://developer.mozilla.org/en-US/docs/Web/Media/Formats/codecs_parameter
            let _source_buffer = media_source
                .add_source_buffer(r#"video/mp4;codecs="avc1.42c01f,mp4a.40.2""#)
                .unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.SourceBuffer.html#method.append_buffer_with_u8_array
            //source_buffer.append_buffer_with_u8_array().unwrap();
            //load video from ipfs
        }
    }

    /* fn destroy(&mut self) {
        bindings::destroy();
    } */
}
