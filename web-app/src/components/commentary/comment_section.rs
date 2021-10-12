use std::collections::HashSet;
use std::rc::Rc;

use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::comments::Comment;
use linked_data::feed::ContentCache;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Page displaying the content of any media.
pub struct CommentSection {
    props: Props,

    comments_cb: Callback<(Cid, Result<Comment>)>,

    comments_set: HashSet<Cid>,
    comments: Vec<(Cid, Rc<str>, Rc<Comment>)>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,

    pub cid: Cid,

    pub content: Rc<ContentCache>,
}

pub enum Msg {
    Comment((Cid, Result<Comment>)),
}

impl Component for CommentSection {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut comp = Self {
            props,

            comments_cb: link.callback(Msg::Comment),

            comments_set: HashSet::with_capacity(10),
            comments: Vec::with_capacity(10),
        };

        comp.get_comments();

        #[cfg(debug_assertions)]
        ConsoleService::info("Comment Section Created");

        comp
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Comment(result) => self.on_comment(result),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&props.content, &self.props.content) {
            self.props = props;

            self.get_comments();
        }

        false
    }

    fn view(&self) -> Html {
        use crate::components::Comment;

        html! {
            <ybc::Section>
                <ybc::Container>
                {
                    for self.comments.iter().rev().map(|(cid, name, comment)| {
                        html! { <Comment cid=*cid name=name.clone() comment=comment.clone() /> }
                    })
                }
                </ybc::Container>
            </ybc::Section>
        }
    }
}

impl CommentSection {
    /// IPFS dag get all comments starting by newest.
    fn get_comments(&mut self) {
        if let Some(iterator) = self.props.content.iter_comments(&self.props.cid) {
            for ipld in iterator {
                if self.comments_set.insert(*ipld) {
                    spawn_local({
                        let ipfs = self.props.ipfs.clone();
                        let cb = self.comments_cb.clone();
                        let cid = *ipld;

                        async move { cb.emit((cid, ipfs.dag_get(cid, Option::<String>::None).await)) }
                    });
                }
            }
        }
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

        let name = match self.props.content.comment_author(&cid) {
            Some(name) => name,
            None => return false,
        };

        let index = self
            .comments
            .binary_search_by_key(&comment.timestamp, |(_, _, probe)| probe.timestamp)
            .unwrap_or_else(|x| x);

        self.comments
            .insert(index, (cid, Rc::from(name), Rc::from(comment)));

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Comments Updated");

        true
    }
}
