use std::rc::Rc;

use crate::components::{
    CommentSection, ExploreCid, IPFSConnectionError, Image, Loading, Markdown, Navbar, VideoPlayer,
};
use crate::utils::{timestamp_to_datetime, IpfsService};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::feed::{ContentCache, Media};
use linked_data::video::VideoMetadata;

use either::Either;

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
    props: Props,

    content_cb: Callback<Result<Media>>,

    state: State,
    author: Rc<str>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,

    pub cid: Cid,

    pub content: Rc<ContentCache>,
}

pub enum Msg {
    Metadata(Result<Media>),
}

impl Component for Content {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut comp = Self {
            props,

            content_cb: link.callback(Msg::Metadata),

            state: State::Loading,
            author: Rc::from(String::default()),
        };

        comp.get_content();

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Page Created");

        comp
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(result) => self.on_metadata(result),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&props.content, &self.props.content) {
            #[cfg(debug_assertions)]
            {
                ConsoleService::info("Content Page Changed");

                ConsoleService::info(&format!("Old {:?}", self.props.content));
                ConsoleService::info(&format!("New {:?}", props.content));
            }

            self.props = props;

            self.get_content();
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Navbar />
                <ybc::Section>
                    <ybc::Container>
                    {
                        match &self.state {
                            State::Loading => html! { <Loading /> },
                            State::Ready(media) => {
                                let dt = timestamp_to_datetime(media.timestamp());

                                match media {
                                Media::Video(video) => self.render_video(dt, video),
                                Media::Blog(blog) => self.render_blog(dt, blog),
                                Media::Statement(twit) => self.render_microblog(dt, twit),
                            }},
                            State::Error => html! { <IPFSConnectionError /> },
                        }
                    }
                    </ybc::Container>
                </ybc::Section>
                <CommentSection ipfs=self.props.ipfs.clone() cid=self.props.cid content=self.props.content.clone() />
            </>
        }
    }
}

impl Content {
    fn render_video(&self, dt: String, metadata: &VideoMetadata) -> Html {
        html! {
            <ybc::Box>
                <ybc::Title>
                    { &metadata.title }
                </ybc::Title>
                <VideoPlayer ipfs=self.props.ipfs.clone() beacon_or_metadata=Either::Right(Rc::from(metadata.clone()))/*TODO find a way to fix this weird clonning issue*/ />
                <ybc::Level>
                    <ybc::LevelLeft>
                        <ybc::LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &*self.author } </span>
                            </span>
                        </ybc::LevelItem>
                        <ybc::LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { dt } </span>
                            </span>
                        </ybc::LevelItem>
                    </ybc::LevelLeft>
                    <ybc::LevelRight>
                        <ybc::LevelItem>
                            <ExploreCid cid=self.props.cid />
                        </ybc::LevelItem>
                    </ybc::LevelRight>
                </ybc::Level>
            </ybc::Box>
        }
    }

    fn render_blog(&self, dt: String, metadata: &FullPost) -> Html {
        html! {
            <ybc::Box>
                <ybc::Title>
                    { &metadata.title }
                </ybc::Title>
                <ybc::Image size=ybc::ImageSize::Is16by9 >
                    <Image image_cid=metadata.image.link ipfs=self.props.ipfs.clone() />
                </ybc::Image>
                <ybc::Level>
                    <ybc::LevelLeft>
                        <ybc::LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &*self.author } </span>
                            </span>
                        </ybc::LevelItem>
                        <ybc::LevelItem>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { dt } </span>
                            </span>
                        </ybc::LevelItem>
                    </ybc::LevelLeft>
                    <ybc::LevelRight>
                        <ybc::LevelItem>
                            <ExploreCid cid=self.props.cid />
                        </ybc::LevelItem>
                    </ybc::LevelRight>
                </ybc::Level>
                <ybc::Content>
                    <Markdown ipfs=self.props.ipfs.clone() markdown_cid=metadata.content.link />
                </ybc::Content>
            </ybc::Box>
        }
    }

    fn render_microblog(&self, dt: String, metadata: &MicroPost) -> Html {
        html! {
            <ybc::Box>
                <ybc::Media>
                    <ybc::MediaLeft>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &*self.author } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { dt } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <ExploreCid cid=self.props.cid />
                        </ybc::Block>
                    </ybc::MediaLeft>
                    <ybc::MediaContent>
                        <ybc::Content classes=classes!("has-text-centered")>
                            { &metadata.content }
                        </ybc::Content>
                    </ybc::MediaContent>
                </ybc::Media>
            </ybc::Box>
        }
    }

    fn get_content(&mut self) {
        spawn_local({
            let cb = self.content_cb.clone();
            let ipfs = self.props.ipfs.clone();
            let cid = self.props.cid;

            async move { cb.emit(ipfs.dag_get(cid, Option::<String>::None).await) }
        });

        self.author = match self.props.content.media_content_author(&self.props.cid) {
            Some(auth) => Rc::from(auth),
            None => Rc::from(String::default()),
        };
    }

    fn on_metadata(&mut self, response: Result<Media>) -> bool {
        self.state = match response {
            Ok(md) => State::Ready(md),
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                State::Error
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Metadata Updated");

        true
    }
}
