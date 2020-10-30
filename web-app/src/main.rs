mod bindings;
mod playlists;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

struct Model {
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        let _s = &self.link;

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <video id="video" width="1280" height="720" autoplay=true controls=true muted=true
                poster="/live_like_poster.png">
                </video>
            </>
        }
    }
}

fn main() {
    bindings::init("livelike");

    yew::start_app::<Model>();
}
