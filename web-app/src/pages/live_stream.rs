use crate::components::{ChatWindow, LiveStreamPlayer};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct LiveStream {}

impl Component for LiveStream {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="live_stream_page">
                <LiveStreamPlayer />
                <ChatWindow />
            </div>
        }
    }
}
