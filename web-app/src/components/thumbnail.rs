use std::rc::Rc;

use crate::app::AppRoute;
use crate::components::{ExploreCid, Image};
use crate::utils::{seconds_to_timecode, timestamp_to_datetime, IpfsService};

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::feed::Media;
use linked_data::video::VideoMetadata;

use cid::Cid;

type Anchor = RouterAnchor<AppRoute>;

/// Content thumbnails for any media type.
#[derive(Properties, Clone)]
pub struct Thumbnail {
    pub cid: Cid,
    pub name: Rc<str>,
    pub metadata: Rc<Media>,
    pub count: usize,
    pub ipfs: IpfsService,
}

impl Component for Thumbnail {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&props.name, &self.name) || !Rc::ptr_eq(&props.metadata, &self.metadata) {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        let metadata = &*self.metadata;

        let dt = timestamp_to_datetime(metadata.timestamp());

        match metadata {
            Media::Video(metadata) => self.render_video(dt, metadata),
            Media::Blog(metadata) => self.render_blog(dt, metadata),
            Media::Statement(metadata) => self.render_statement(dt, metadata),
        }
    }
}

impl Thumbnail {
    fn render_video(&self, dt: String, metadata: &VideoMetadata) -> Html {
        let (hour, minute, second) = seconds_to_timecode(metadata.duration);

        html! {
            <ybc::Box>
                <ybc::Media>
                    <ybc::MediaLeft>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &self.name } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { dt } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-video"></i></span>
                                <span> { &format!("{}:{}:{}", hour, minute, second) } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-comments"></i></span>
                                <span> { &format!("{} Comment", self.count) } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <ExploreCid cid=self.cid />
                        </ybc::Block>
                    </ybc::MediaLeft>
                        <ybc::MediaContent classes=classes!("has-text-centered") >
                            <Anchor route=AppRoute::Content(self.cid)>
                                <ybc::Title classes=classes!("is-6") >
                                    { &metadata.title }
                                </ybc::Title>
                                <ybc::Image size=ybc::ImageSize::Is16by9 >
                                    <Image image_cid=metadata.image.link ipfs=self.ipfs.clone() />
                                </ybc::Image>
                            </Anchor>
                        </ybc::MediaContent>
                </ybc::Media>
            </ybc::Box>
        }
    }

    fn render_blog(&self, dt: String, metadata: &FullPost) -> Html {
        html! {
            <ybc::Box>
                <ybc::Media>
                    <ybc::MediaLeft>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &self.name } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { dt } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-comments"></i></span>
                                <span> { &format!("{} Comment", self.count) } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <ExploreCid cid=self.cid />
                        </ybc::Block>
                    </ybc::MediaLeft>
                    <ybc::MediaContent classes=classes!("has-text-centered") >
                        <Anchor route=AppRoute::Content(self.cid)>
                            <ybc::Title classes=classes!("is-6") >
                                { &metadata.title }
                            </ybc::Title>
                            <ybc::Image size=ybc::ImageSize::Is16by9 >
                                <Image image_cid=metadata.image.link ipfs=self.ipfs.clone() />
                            </ybc::Image>
                        </Anchor>
                    </ybc::MediaContent>
                </ybc::Media>
            </ybc::Box>
        }
    }

    fn render_statement(&self, dt: String, metadata: &MicroPost) -> Html {
        html! {
            <ybc::Box>
                <ybc::Media>
                    <ybc::MediaLeft>
                            <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-user"></i></span>
                                <span> { &self.name } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-clock"></i></span>
                                <span> { dt } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <span class="icon-text">
                                <span class="icon"><i class="fas fa-comments"></i></span>
                                <span> { &format!("{} Comment", self.count) } </span>
                            </span>
                        </ybc::Block>
                        <ybc::Block>
                            <ExploreCid cid=self.cid />
                        </ybc::Block>
                    </ybc::MediaLeft>
                    <ybc::MediaContent>
                        <Anchor route=AppRoute::Content(self.cid)>
                            <ybc::Subtitle classes=classes!("has-text-centered") size=ybc::HeaderSize::Is6 >
                                { &metadata.content }
                            </ybc::Subtitle>
                        </Anchor>
                    </ybc::MediaContent>
                </ybc::Media>
            </ybc::Box>
        }
    }
}
