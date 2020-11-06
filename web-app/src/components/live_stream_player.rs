use crate::bindings;
use crate::live_stream_manager::LiveStreamManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct LiveStreamPlayer {
    _live_stream: LiveStreamManager,
}

impl Component for LiveStreamPlayer {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let live_stream = LiveStreamManager::new();

        live_stream.playlists_updating();

        live_stream.register_callback();

        Self {
            _live_stream: live_stream,
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
