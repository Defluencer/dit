use crate::agents::VideoOnDemandManager;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::beacon::VideoMetaData;

pub struct VideoPlayer {
    manager: VideoOnDemandManager,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub metadata: VideoMetaData,
}

impl Component for VideoPlayer {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            manager: VideoOnDemandManager::new(props.metadata),
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
            <video id="video" controls=true />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.manager.link_video();
        }
    }
}
