use crate::app::AppRoute;
use crate::components::seconds_to_timecode;
use crate::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew_router::components::RouterAnchor;

use linked_data::video::VideoMetadata;

use cid::Cid;

type Anchor = RouterAnchor<AppRoute>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct VideoThumbnail {
    props: Props,

    metadata: VideoMetadata,
}

pub enum Msg {
    Metadata(Result<VideoMetadata>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub metadata_cid: Cid,
}

impl Component for VideoThumbnail {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback_once(Msg::Metadata);
        let client = props.ipfs.clone();
        let cid = props.metadata_cid;

        spawn_local(async move { cb.emit(client.dag_get(cid, Option::<String>::None).await) });

        Self {
            props,

            metadata: VideoMetadata::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(result) => self.on_video_metadata_update(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let (hour, minute, second) = seconds_to_timecode(self.metadata.duration);

        html! {
            <div class="video_thumbnail">
                <Anchor route=AppRoute::Video(self.props.metadata_cid) classes="thumbnail_link">
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

impl VideoThumbnail {
    /// Callback when IPFS dag get returns VideoMetadata node.
    fn on_video_metadata_update(&mut self, response: Result<VideoMetadata>) -> bool {
        let metadata = match response {
            Ok(metadata) => metadata,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if self.metadata == metadata {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Video Metadata Update");

        self.metadata = metadata;

        true
    }
}
