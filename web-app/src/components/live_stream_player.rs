use crate::agents::LiveStream;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, SourceBuffer, Url, Window};

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct LiveStreamPlayer {
    pub stream: LiveStream,
}

impl Component for LiveStreamPlayer {
    type Message = ();
    type Properties = ();

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let topic = "livelikevideo";

        let streamer_peer_id = "12D3KooWAPZ3QZnZUJw3BgEX9F7XL383xFNiKQ5YKANiRC3NWvpo";

        let stream = LiveStream::new(topic, streamer_peer_id);

        Self { stream }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <video id="video" autoplay=true controls=true muted=true poster="../live_like_poster.png" />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.stream.link_video();
        }
    }

    //fn destroy(&mut self) {}
}
