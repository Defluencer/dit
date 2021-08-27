use std::rc::Rc;

use crate::app::AppRoute;
use crate::components::Image;
use crate::utils::seconds_to_timecode;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

use linked_data::blog::{FullPost, MicroPost};
use linked_data::feed::Media;
use linked_data::video::VideoMetadata;

use cid::Cid;

type Anchor = RouterAnchor<AppRoute>;

/// Content thumbnails for any media type.
#[derive(Properties, Clone, PartialEq)]
pub struct Thumbnail {
    pub cid: Cid,
    pub name: Rc<str>,
    pub metadata: Rc<Media>,
    pub count: usize,
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
        if *self != props {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        match &*self.metadata {
            Media::Video(metadata) => self.render_video(metadata),
            Media::Blog(metadata) => self.render_blog(metadata),
            Media::Statement(metadata) => self.render_statement(metadata),
        }
    }
}

impl Thumbnail {
    fn render_video(&self, metadata: &VideoMetadata) -> Html {
        let (hour, minute, second) = seconds_to_timecode(metadata.duration);

        html! {
            <div class="thumbnail">
                <div class="thumbnail_author"> { &self.name } </div>
                <Anchor route=AppRoute::Content(self.cid) classes="thumbnail_link">
                    <div class="video_thumbnail_title"> { &metadata.title } </div>
                    <div class="video_thumbnail_image">
                        <Image image_cid=metadata.image.link />
                    </div>
                    <div class="video_thumbnail_duration"> { &format!("{}:{}:{}", hour, minute, second) } </div>
                    <div class="comment_count"> { &format!("{} Comments", self.count) } </div>
                </Anchor>
            </div>
        }
    }

    fn render_blog(&self, metadata: &FullPost) -> Html {
        html! {
            <div class="thumbnail">
                <div class="thumbnail_author"> { &self.name } </div>
                <Anchor route=AppRoute::Content(self.cid) classes="thumbnail_link">
                    <div class="post_thumbnail_title"> { &metadata.title } </div>
                    <div class="post_thumbnail_image">
                        <Image image_cid=metadata.image.link />
                    </div>
                    <div class="comment_count"> { &format!("{} Comments", self.count) } </div>
                </Anchor>
            </div>
        }
    }

    fn render_statement(&self, metadata: &MicroPost) -> Html {
        html! {
            <div class="thumbnail">
                <div class="thumbnail_author"> { &self.name } </div>
                <Anchor route=AppRoute::Content(self.cid) classes="thumbnail_link">
                    <div class="statement_text"> { &metadata.content } </div>
                    <div class="comment_count"> { &format!("{} Comments", self.count) } </div>
                </Anchor>
            </div>
        }
    }
}
