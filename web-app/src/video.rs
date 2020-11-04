use crate::live_stream::LiveStreamManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::Properties;

pub struct LiveStreamPlayer {
    _link: ComponentLink<Self>,

    props: LiveStreamProps,
}

#[derive(Properties, Clone)]
pub struct LiveStreamProps {
    pub manager: LiveStreamManager,
}

impl Component for LiveStreamPlayer {
    type Message = ();
    type Properties = LiveStreamProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { _link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <video id="video" inline=true muted=true controls=true poster="/live_like_poster.png">
                </video>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.props.manager.init_hls();
        }
    }
}
