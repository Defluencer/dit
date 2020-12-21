use crate::agents::LiveStreamManager;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(Clone, Debug, Properties)]
pub struct LiveStreamData {
    pub topic: String,

    pub streamer_peer_id: String,
}

pub struct LiveStreamPlayer {
    pub stream: LiveStreamManager,
}

impl Component for LiveStreamPlayer {
    type Message = ();
    type Properties = LiveStreamData;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            stream: LiveStreamManager::new(props.topic, props.streamer_peer_id),
        }
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
}
