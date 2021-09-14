use std::collections::HashSet;
use std::rc::Rc;

use crate::components::{Error, Image, Loading, Markdown, Navbar, VideoPlayer};
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::comments::Comment;
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
    comments_cb: Callback<(Cid, Result<Comment>)>,

    state: State,
    author: Rc<str>,

    comments_set: HashSet<Cid>,
    comments: Vec<(Rc<str>, Rc<Comment>)>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,

    pub cid: Cid,

    pub content: Rc<ContentCache>,
}

pub enum Msg {
    Metadata(Result<Media>),
    Comment((Cid, Result<Comment>)),
}

impl Component for Content {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut comp = Self {
            props,

            content_cb: link.callback(Msg::Metadata),
            comments_cb: link.callback(Msg::Comment),

            state: State::Loading,
            author: Rc::from(String::default()),

            comments_set: HashSet::with_capacity(10),
            comments: Vec::with_capacity(10),
        };

        comp.get_content();
        comp.get_comments();

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Page Created");

        comp
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(result) => self.on_metadata(result),
            Msg::Comment(result) => self.on_comment(result),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&props.content, &self.props.content) {
            #[cfg(debug_assertions)]
            ConsoleService::info("Content Page Changed");

            ConsoleService::info(&format!("Old {:?}", self.props.content));
            ConsoleService::info(&format!("New {:?}", props.content));

            self.props = props;

            self.get_content();
            self.get_comments();
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Navbar />
                <ybc::Section>
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
                </ybc::Section>
                <ybc::Section>
                    <ybc::Container>
                    {
                        for self.comments.iter().rev().map(|(name, comment)| {
                            html! { <crate::components::Comment name=name.clone() comment=comment.clone() /> }
                        })
                    }
                    </ybc::Container>
                </ybc::Section>
            </>
        }
    }
}

impl Content {
    fn render_video(&self, metadata: &VideoMetadata) -> Html {
        html! {
            <ybc::Container>
                <ybc::Box classes=classes!("has-text-centered")>
                    <ybc::Title>
                        { &metadata.title }
                    </ybc::Title>
                    <VideoPlayer ipfs=self.props.ipfs.clone() beacon_or_metadata=Either::Right(Rc::from(metadata.clone()))/*TODO find a way to fix this weird clonning issue*/ />
                </ybc::Box>
            </ybc::Container>
        }
    }

    fn render_blog(&self, metadata: &FullPost) -> Html {
        html! {
            <ybc::Container>
                <ybc::Box classes=classes!("has-text-centered") >
                    <ybc::Title>
                        { &metadata.title }
                    </ybc::Title>
                    <ybc::Image size=ybc::ImageSize::Is16by9 >
                        <Image image_cid=metadata.image.link />
                    </ybc::Image>
                </ybc::Box>
                <ybc::Box>
                    <ybc::Content>
                        <Markdown ipfs=self.props.ipfs.clone() markdown_cid=metadata.content.link />
                    </ybc::Content>
                </ybc::Box>
            </ybc::Container>
        }
    }

    fn render_microblog(&self, metadata: &MicroPost) -> Html {
        html! {
            <ybc::Container>
                <ybc::Box>
                    <ybc::Media>
                        <ybc::MediaLeft>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &*self.author } </span>
                            </span>
                        </ybc::MediaLeft>
                        <ybc::MediaContent>
                            <ybc::Content classes=classes!("has-text-centered")>
                                { &metadata.content }
                            </ybc::Content>
                        </ybc::MediaContent>
                    </ybc::Media>
                </ybc::Box>
            </ybc::Container>
        }
    }

    fn get_content(&mut self) {
        spawn_local({
            let cb = self.content_cb.clone();
            let ipfs = self.props.ipfs.clone();
            let cid = self.props.cid;

            async move { cb.emit(ipfs.dag_get(cid, Option::<String>::None).await) }
        });

        self.author = match self.props.content.get_content_author(&self.props.cid) {
            Some(auth) => Rc::from(auth),
            None => Rc::from(String::default()),
        };
    }

    /// IPFS dag get all comments starting by newest.
    fn get_comments(&mut self) {
        for ipld in self.props.content.iter_comments(&self.props.cid) {
            if self.comments_set.insert(ipld.link) {
                spawn_local({
                    let ipfs = self.props.ipfs.clone();
                    let cb = self.comments_cb.clone();
                    let cid = ipld.link;

                    async move { cb.emit((cid, ipfs.dag_get(cid, Option::<String>::None).await)) }
                });
            }
        }
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

    fn on_comment(&mut self, response: (Cid, Result<Comment>)) -> bool {
        let (cid, comment) = match response {
            (cid, Ok(node)) => (cid, node),
            (_, Err(e)) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        /* if !signed_comment.verify() {
            return false;
        } */

        let name = match self.props.content.get_comment_author(&cid) {
            Some(name) => name,
            None => return false,
        };

        let index = self
            .comments
            .binary_search_by(|(_, probe)| probe.timestamp.cmp(&comment.timestamp))
            .unwrap_or_else(|x| x);

        self.comments
            .insert(index, (Rc::from(name), Rc::from(comment)));

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Comments Updated");

        true
    }
}
