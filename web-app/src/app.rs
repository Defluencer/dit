use crate::buttons::PlayButton;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct App {
    _link: ComponentLink<Self>,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { _link }
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
                <video id="video" width="1280" height="720" muted=true
                poster="/live_like_poster.png">
                </video>

                <PlayButton />
            </>
        }
    }
}
