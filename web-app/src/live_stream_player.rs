#![allow(dead_code, unused_variables)]

use crate::bindings;
use crate::live_stream_manager::LiveStreamManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::Properties;

pub struct LiveStreamPlayer {
    link: ComponentLink<Self>,

    props: LiveStreamProps,
}

#[derive(Properties, Clone)]
pub struct LiveStreamProps {
    pub live_stream: LiveStreamManager,
}

impl Component for LiveStreamPlayer {
    type Message = ();
    type Properties = LiveStreamProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        props.live_stream.register_callback();

        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
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
            bindings::attach_media(); // Must be called after <video> is added
        }
    }

    fn destroy(&mut self) {
        bindings::destroy();
    }
}
