use crate::components::{ChatWindow, LiveStreamPlayer};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

use linked_data::{LIVE_VIDEO_TOPIC, STREAMER_PEER_ID};

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
                <LiveStreamPlayer topic=LIVE_VIDEO_TOPIC streamer_peer_id=STREAMER_PEER_ID />
                <ChatWindow />
            </div>
        }
    }
}
