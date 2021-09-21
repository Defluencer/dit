use std::collections::HashSet;
use std::rc::Rc;

use crate::components::{Loading, Navbar, Thumbnail};
use crate::utils::{IpfsService, LocalStorage};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::{Callback, MouseEvent};

use linked_data::feed::{ContentCache, Media};

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(PartialEq)]
pub enum FilterType {
    None,
    Videos,
    Blogs,
    Statements,
}

/// Page displaying content thumbnails from you and your friends.
pub struct ContentFeed {
    props: Props,

    media_cb: Callback<(Cid, Result<Media>)>,
    content_set: HashSet<Cid>,
    content: Vec<(Cid, Rc<str>, Rc<Media>, usize)>,

    no_filter_cb: Callback<MouseEvent>,
    vid_filter_cb: Callback<MouseEvent>,
    blog_filter_cb: Callback<MouseEvent>,
    micro_filter_cb: Callback<MouseEvent>,
    filter: FilterType,
}

pub enum Msg {
    Metadata((Cid, Result<Media>)),
    Filter(FilterType),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub storage: LocalStorage,
    pub content: Rc<ContentCache>,
}

impl Component for ContentFeed {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut feed = Self {
            props,

            media_cb: link.callback(Msg::Metadata),
            content_set: HashSet::with_capacity(100),
            content: Vec::with_capacity(100),

            no_filter_cb: link.callback(|_event: MouseEvent| Msg::Filter(FilterType::None)),
            vid_filter_cb: link.callback(|_event: MouseEvent| Msg::Filter(FilterType::Videos)),
            blog_filter_cb: link.callback(|_event: MouseEvent| Msg::Filter(FilterType::Blogs)),
            micro_filter_cb: link
                .callback(|_event: MouseEvent| Msg::Filter(FilterType::Statements)),
            filter: FilterType::None,
        };

        feed.get_content();

        feed
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(result) => self.on_metadata(result),
            Msg::Filter(filter) => {
                if self.filter != filter {
                    self.filter = filter;

                    return true;
                }

                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&props.content, &self.props.content) {
            self.props = props;

            self.get_content();
        }

        false
    }

    fn view(&self) -> Html {
        let content = if self.content.is_empty() {
            html! {  <Loading /> }
        } else {
            self.render_thumbnails()
        };

        html! {
            <>
                <Navbar />
                <ybc::Section>
                    <ybc::Container>
                        <ybc::Tabs classes=classes!("is-small") toggle=true fullwidth=true >
                            <li class={if let FilterType::None = self.filter {"is-active"} else {""}} >
                                <a onclick=self.no_filter_cb.clone() >
                                    <span class="icon-text">
                                        <span class="icon"><i class="fas fa-stream"></i></span>
                                        <span> { "No Filter" } </span>
                                    </span>
                                </a>
                            </li>
                            <li class={if let FilterType::Videos = self.filter {"is-active"} else {""}} >
                                <a onclick=self.vid_filter_cb.clone() >
                                    <span class="icon-text">
                                        <span class="icon"><i class="fas fa-video"></i></span>
                                        <span> { "Videos" } </span>
                                    </span>
                                </a>
                            </li>
                            <li class={if let FilterType::Blogs = self.filter {"is-active"} else {""}} >
                                <a onclick=self.blog_filter_cb.clone() >
                                    <span class="icon-text">
                                        <span class="icon"><i class="fas fa-blog"></i></span>
                                        <span> { "Blogs" } </span>
                                    </span>
                                </a>
                            </li>
                            <li class={if let FilterType::Statements = self.filter {"is-active"} else {""}} >
                                <a onclick=self.micro_filter_cb.clone() >
                                    <span class="icon-text">
                                        <span class="icon"><i class="fas fa-comment"></i></span>
                                        <span> { "Statements" } </span>
                                    </span>
                                </a>
                            </li>
                        </ybc::Tabs>
                        { content }
                    </ybc::Container>
                </ybc::Section>
            </>
        }
    }
}

impl ContentFeed {
    fn render_thumbnails(&self) -> Html {
        html! {
            <>
            {
            self.content.iter().rev().filter_map(|(cid, name, metadata, count)| {
                match (metadata.as_ref(), &self.filter) {
                    (_, FilterType::None) => Some(html! { <Thumbnail cid=*cid name=name.clone()  metadata=metadata.clone() count=*count ipfs=self.props.ipfs.clone() />}),
                    (Media::Video(_), FilterType::Videos) => Some(html! { <Thumbnail cid=*cid name=name.clone()  metadata=metadata.clone() count=*count ipfs=self.props.ipfs.clone() />}),
                    (Media::Blog(_), FilterType::Blogs) => Some(html! { <Thumbnail cid=*cid name=name.clone()  metadata=metadata.clone() count=*count ipfs=self.props.ipfs.clone() />}),
                    (Media::Statement(_), FilterType::Statements) => Some(html! { <Thumbnail cid=*cid name=name.clone()  metadata=metadata.clone() count=*count ipfs=self.props.ipfs.clone() />}),
                    (_, _) => None,
                }
            }).collect::<Html>()
            }
            </>
        }
    }

    /// IPFS dag get all metadata from content feed starting by newest.
    fn get_content(&mut self) {
        for cid in self.props.content.iter_content() {
            if self.content_set.insert(*cid) {
                spawn_local({
                    let cb = self.media_cb.clone();
                    let ipfs = self.props.ipfs.clone();
                    let cid = *cid;

                    async move { cb.emit((cid, ipfs.dag_get(cid, Option::<&str>::None).await)) }
                });
            }
        }
    }

    /// Callback when IPFS dag get returns a Media node.
    fn on_metadata(&mut self, response: (Cid, Result<Media>)) -> bool {
        let (cid, metadata) = match response {
            (cid, Ok(metadata)) => (cid, metadata),
            (_, Err(e)) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        let index = self
            .content
            .binary_search_by(|(_, _, probe, _)| probe.timestamp().cmp(&metadata.timestamp()))
            .unwrap_or_else(|x| x);

        let name = match self.props.content.get_content_author(&cid) {
            Some(name) => name,
            None => return false,
        };

        let count = self.props.content.get_comments_count(&cid);

        self.content
            .insert(index, (cid, Rc::from(name), Rc::from(metadata), count));

        #[cfg(debug_assertions)]
        ConsoleService::info("Feed Metadata Updated");

        true
    }
}
