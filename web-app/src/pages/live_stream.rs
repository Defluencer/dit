use crate::components::{ChatWindow, LiveStreamPlayer};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct LiveStream {}

impl Component for LiveStream {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <LiveStreamPlayer />
                <ChatWindow />
                {"Video On Demand Page -> W.I.P."}
            </div>
        }
    }
}
