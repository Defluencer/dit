use crate::agents::VideoOnDemandManager;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(Clone, Debug, Properties)]
pub struct VideoData {
    pub title: String,

    pub duration: f64,

    pub video_cid: String,
    //TODO thumbnail img cid
}

pub struct VideoPlayer {
    manager: VideoOnDemandManager,
}

impl Component for VideoPlayer {
    type Message = ();
    type Properties = VideoData;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            manager: VideoOnDemandManager::new(props.video_cid, props.duration),
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
            <video id="video" controls=true poster="../live_like_poster.png" />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.manager.link_video();
        }
    }
}
