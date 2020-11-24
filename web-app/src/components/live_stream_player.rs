use crate::agents::{load_live_stream, unload_live_stream};
use crate::bindings;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct LiveStreamPlayer {}

impl Component for LiveStreamPlayer {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        load_live_stream();

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
            <video id="video" muted=true controls=true poster="../live_like_poster.png">
            </video>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            bindings::hls_attach_media(); // Must be called after <video> element is rendered
        }
    }

    fn destroy(&mut self) {
        unload_live_stream();
    }
}
