use std::rc::Rc;

use crate::components::{Error, Image, Loading, Markdown, Navbar, VideoPlayer};
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::feed::Media;
use linked_data::video::VideoMetadata;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[allow(clippy::large_enum_variant)]
enum State {
    Loading,
    Ready(Media),
    Error,
}

/// Page displaying the content of any media.
pub struct Content {
    ipfs: IpfsService,
    state: State,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,
    pub metadata_cid: Cid,
}

pub enum Msg {
    Metadata(Result<Media>),
}

impl Component for Content {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, metadata_cid } = props;

        spawn_local({
            let cb = link.callback_once(Msg::Metadata);
            let ipfs = ipfs.clone();

            async move { cb.emit(ipfs.dag_get(metadata_cid, Option::<String>::None).await) }
        });

        Self {
            ipfs,
            state: State::Loading,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(result) => self.update_metadata(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="content_page">
            <Navbar />
            {
                match &self.state {
                    State::Loading => html! { <Loading /> },
                    State::Ready(media) => match media {
                        Media::Video(video) => self.render_video(video),
                        Media::Blog(blog) => self.render_blog(blog),
                        Media::Statement(twit) => self.render_microblog(twit),
                    },
                    State::Error => html! { <Error /> },
                }
            }
            </div>
        }
    }
}

impl Content {
    fn update_metadata(&mut self, response: Result<Media>) -> bool {
        self.state = match response {
            Ok(md) => State::Ready(md),
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                State::Error
            }
        };

        true
    }

    fn render_blog(&self, metadata: &FullPost) -> Html {
        html! {
            <div class="blog">
                <div class="post_title"> { &metadata.title } </div>
                <div class="post_image">
                    <Image image_cid=metadata.image.link />
                </div>
                <div class="post_content">
                    <Markdown ipfs=self.ipfs.clone() markdown_cid=metadata.content.link />
                </div>
            </div>
        }
    }

    fn render_video(&self, metadata: &VideoMetadata) -> Html {
        html! {
            <div class="video">
                <VideoPlayer ipfs=self.ipfs.clone() metadata=Rc::from(metadata.clone())/*TODO find a way to fix this weird clonning issue*/ />
            </div>
        }
    }

    fn render_microblog(&self, metadata: &MicroPost) -> Html {
        html! {
            <div class="micro_blog">
                <div class="post_content"> { &metadata.content } </div>
            </div>
        }
    }
}
