use crate::app::AppRoute;
use crate::components::seconds_to_timecode;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

use yewtil::NeqAssign;

use linked_data::video::VideoMetadata;

use cid::Cid;

type Anchor = RouterAnchor<AppRoute>;

#[derive(PartialEq, Clone, Properties)]
pub struct VideoThumbnail {
    pub metadata_cid: Cid,
    pub metadata: VideoMetadata,
}

impl Component for VideoThumbnail {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.neq_assign(props)
    }

    fn view(&self) -> Html {
        let (hour, minute, second) = seconds_to_timecode(self.metadata.duration);

        html! {
            <div class="video_thumbnail">
                <Anchor route=AppRoute::Video(self.metadata_cid) classes="thumbnail_link">
                    <div class="thumbnail_title"> {&self.metadata.title} </div>
                    <div class="thumbnail_image">
                        <img src=format!("ipfs://{}", &self.metadata.image.link.to_string()) alt="This image require IPFS native browser" />
                    </div>
                    <div class="thumbnail_duration"> {&format!("{}:{}:{}", hour, minute, second) } </div>
                </Anchor>
            </div>
        }
    }
}
