use std::collections::HashSet;
use std::rc::Rc;

use crate::components::{Comment, Error, Image, Loading, Markdown, Navbar, VideoPlayer};
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::comments::Commentary;
use linked_data::feed::Media;
use linked_data::signature::SignedMessage;
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

    cb: Callback<Result<SignedMessage<linked_data::comments::Comment>>>,

    state: State,

    comments_set: HashSet<Cid>,
    comments: Vec<Rc<SignedMessage<linked_data::comments::Comment>>>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,

    pub metadata_cid: Cid,

    pub comments: Rc<Commentary>,
}

pub enum Msg {
    Metadata(Result<Media>),
    Comment(Result<SignedMessage<linked_data::comments::Comment>>),
}

impl Component for Content {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        spawn_local({
            let cb = link.callback_once(Msg::Metadata);
            let ipfs = props.ipfs.clone();
            let cid = props.metadata_cid;

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
        if props.comments != self.props.comments {
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
                    for self.comments.iter().rev().map(|comment| {
                        html! { <Comment signed_comment=comment.clone() /> }
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
        let cid = self.props.metadata_cid.to_string();

        if let Some(vec) = self.props.comments.map.get(&cid) {
            for ipld in vec.iter().rev() {
                if self.comments_set.insert(ipld.link) {
                    spawn_local({
                        let ipfs = self.props.ipfs.clone();
                        let cb = self.cb.clone();
                        let cid = ipld.link;

                        async move { cb.emit(ipfs.dag_get(cid, Option::<String>::None).await) }
                    });
                }
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

    fn on_comment(
        &mut self,
        response: Result<SignedMessage<linked_data::comments::Comment>>,
    ) -> bool {
        let signed_comment = match response {
            Ok(data) => data,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Comment Update");

        if !signed_comment.verify() {
            return false;
        }

        let index = self
            .comments
            .binary_search_by(|probe| probe.data.timestamp.cmp(&signed_comment.data.timestamp))
            .unwrap_or_else(|x| x);

        self.comments.insert(index, Rc::from(signed_comment));

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
