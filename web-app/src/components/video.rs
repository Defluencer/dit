use crate::agents::load_video;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Video {
    //title

    //duration

    //thumbnail cid
    pub video_cid: String,
}

impl Component for Video {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <video id="video" controls=true poster="../live_like_poster.png" />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            load_video(self.video_cid.clone(), 90.0);
        }
    }
}
