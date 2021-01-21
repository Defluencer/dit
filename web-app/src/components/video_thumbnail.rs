use crate::app::Route;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

use yewtil::NeqAssign;

use linked_data::beacon::VideoMetadata;

use cid::Cid;

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
        unimplemented!()
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.neq_assign(props)
    }

    fn view(&self) -> Html {
        type Anchor = RouterAnchor<Route>;

        html! {
            <div class="video_thumbnail">
                <Anchor route=Route::Video(self.metadata_cid) classes="thumbnail_link">
                    <div class="thumbnail_title"> {&self.metadata.title} </div>
                    <div class="thumbnail_image">
                        //<img src=format!("ipfs://{}", &self.metadata.image.link.to_string()) alt=&self.metadata.title />
                        //<img src="/live_like_poster.png" alt=&self.metadata.title />
                    </div>
                    <div class="thumbnail_duration"> {&self.metadata.duration} </div>
                </Anchor>
            </div>
        }
    }
}
