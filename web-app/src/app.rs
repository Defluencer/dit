#![allow(dead_code, unused_variables)]

use crate::live_stream_manager::LiveStreamManager;
use crate::live_stream_player::LiveStreamPlayer;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct App {
    link: ComponentLink<Self>,

    live_stream: LiveStreamManager,
}

//TODO add router & tabs

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let live_stream = LiveStreamManager::new();

        live_stream.playlists_updating();

        Self { link, live_stream }
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
                <LiveStreamPlayer live_stream=self.live_stream.clone() />
            </>
        }
    }
}
