use crate::live_stream::LiveStreamManager;
use crate::video::LiveStreamPlayer;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct App {
    _link: ComponentLink<Self>,

    manager: LiveStreamManager,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let manager = LiveStreamManager::new();

        Self { _link, manager }
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
                <LiveStreamPlayer manager=self.manager.clone() />
            </>
        }
    }

    /* fn rendered(&mut self, first_render: bool) {
        if first_render {}
    } */
}
