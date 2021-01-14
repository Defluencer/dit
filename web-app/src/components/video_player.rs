use crate::agents::VideoOnDemandManager;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::beacon::VideoMetadata;

pub struct VideoPlayer {
    manager: VideoOnDemandManager,
    poster_link: String,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub metadata: VideoMetadata,
}

impl Component for VideoPlayer {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let mut poster_link = String::from("ipfs:/");
        poster_link.push_str(&props.metadata.image.link.to_string());

        let manager = VideoOnDemandManager::new(props.metadata);

        Self {
            manager,
            poster_link,
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
            <video id="video" controls=true poster=self.poster_link />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.manager.link_video();
        }
    }
}
