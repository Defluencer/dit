use std::collections::HashSet;
use std::rc::Rc;

use crate::components::{Error, Image, Loading, Markdown, Navbar, VideoPlayer};
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::comments::Comment;
use linked_data::feed::{ContentCache, Media};
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
    props: Props,

    cb: Callback<(Cid, Result<Comment>)>,

    state: State,

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
        spawn_local({
            let cb = link.callback_once(Msg::Metadata);
            let ipfs = props.ipfs.clone();
            let cid = props.cid;

            async move { cb.emit(ipfs.dag_get(cid, Option::<String>::None).await) }
        });

        let mut content = Self {
            props,

            cb: link.callback(Msg::Comment),

            state: State::Loading,

            comments_set: HashSet::with_capacity(10),
            comments: Vec::with_capacity(10),
        };

        content.get_comments();

        content
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(result) => self.on_metadata(result),
            Msg::Comment(result) => self.on_comment(result),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.content != self.props.content {
            self.props = props;

            self.get_comments();
        }

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
                <div class="comment_section">
                {
                    for self.comments.iter().rev().map(|(name, comment)| {
                        html! { <crate::components::Comment name=name.clone() comment=comment.clone() /> }
                    })
                }
                </div>
            </div>
        }
    }
}

impl Content {
    /// IPFS dag get all comments starting by newest.
    fn get_comments(&mut self) {
        for ipld in self.props.content.iter_comments(&self.props.cid) {
            if self.comments_set.insert(ipld.link) {
                spawn_local({
                    let ipfs = self.props.ipfs.clone();
                    let cb = self.cb.clone();
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

        #[cfg(debug_assertions)]
        ConsoleService::info("Comment Update");

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
                    <Markdown ipfs=self.props.ipfs.clone() markdown_cid=metadata.content.link />
                </div>
            </div>
        }
    }

    fn render_video(&self, metadata: &VideoMetadata) -> Html {
        html! {
            <div class="video">
                <VideoPlayer ipfs=self.props.ipfs.clone() metadata=Rc::from(metadata.clone())/*TODO find a way to fix this weird clonning issue*/ />
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
