use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

use crate::components::Video;

pub struct VideoOnDemand {}

impl Component for VideoOnDemand {
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
            <div class="vod_page">
                <Video video_cid="bafyreibndv7uudvdpimdxgtm6dacrla7r2z6qd34c76x5bl74fv6fhu4sy" />
            </div>
        }
    }
}
