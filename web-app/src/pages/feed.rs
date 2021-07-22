use std::rc::Rc;

use crate::components::{Navbar, Thumbnail};
use crate::utils::{IpfsService, LocalStorage};

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::feed::FeedAnchor;

#[derive(Properties, Clone)]
pub struct ContentFeed {
    pub ipfs: IpfsService,
    pub storage: LocalStorage,
    pub feed: Rc<FeedAnchor>,
}

impl Component for ContentFeed {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.feed != self.feed {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="content_feed_page">
                <Navbar />
                <div class="feed">
                {
                    for self.feed.content.iter().rev().map(|ipld| {
                        html! {
                            <Thumbnail ipfs=self.ipfs.clone()  metadata_cid=ipld.link />
                        }
                    }
                    )
                }
                </div>
            </div>
        }
    }
}
