use crate::app::AppRoute;
use crate::utils::{seconds_to_timecode, IpfsService};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew_router::components::RouterAnchor;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::feed::Media;
use linked_data::video::VideoMetadata;

use cid::Cid;

type Anchor = RouterAnchor<AppRoute>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Thumbnail {
    props: Props,

    metadata: Media,
    loading: bool,
}

pub enum Msg {
    Metadata(Result<Media>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub metadata_cid: Cid,
}

impl Component for Thumbnail {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        spawn_local({
            let cb = link.callback_once(Msg::Metadata);
            let ipfs = props.ipfs.clone();
            let cid = props.metadata_cid;

            async move { cb.emit(ipfs.dag_get(cid, Option::<&str>::None).await) }
        });

        Self {
            props,

            metadata: Media::default(),
            loading: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(result) => self.on_metadata_update(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if self.loading {
            return html! {
               <> { "Loading..." } </>
            };
        }

        match &self.metadata {
            Media::Video(metadata) => self.render_video(metadata),
            Media::Blog(metadata) => self.render_blog(metadata),
            Media::Statement(metadata) => self.render_statement(metadata),
        }
    }
}

impl Thumbnail {
    /// Callback when IPFS dag get returns Media node.
    fn on_metadata_update(&mut self, response: Result<Media>) -> bool {
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
        ConsoleService::info("Metadata Update");

        self.metadata = metadata;
        self.loading = false;

        true
    }

    fn render_video(&self, metadata: &VideoMetadata) -> Html {
        let (hour, minute, second) = seconds_to_timecode(metadata.duration);

        html! {
            <div class="thumbnail">
                <Anchor route=AppRoute::Video(self.props.metadata_cid) classes="thumbnail_link">
                    <div class="video_thumbnail_title"> { &metadata.title } </div>
                    <div class="video_thumbnail_image">
                        <img src=format!("ipfs://{}", metadata.image.link.to_string()) alt="This image require IPFS native browser" />
                    </div>
                    <div class="video_thumbnail_duration"> {&format!("{}:{}:{}", hour, minute, second) } </div>
                </Anchor>
            </div>
        }
    }

    fn render_blog(&self, metadata: &FullPost) -> Html {
        html! {
            <div class="thumbnail">
                <Anchor route=AppRoute::Blog(self.props.metadata_cid) classes="thumbnail_link">
                    <div class="post_thumbnail_title"> { &metadata.title } </div>
                    <div class="post_thumbnail_image">
                        <img src=format!("ipfs://{}", metadata.image.link.to_string()) alt="This image require IPFS native browser" />
                    </div>
                </Anchor>
            </div>
        }
    }

    fn render_statement(&self, metadata: &MicroPost) -> Html {
        html! {
            <div class="thumbnail">
                <div class="statement_text"> { &metadata.content } </div>
            </div>
        }
    }
}
