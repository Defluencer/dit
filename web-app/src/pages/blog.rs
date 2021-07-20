use crate::components::{Error, Loading, Markdown, Navbar};
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::blog::FullPost;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[allow(clippy::large_enum_variant)]
enum State {
    Loading,
    Ready(FullPost),
    Error,
}

pub struct Blog {
    ipfs: IpfsService,
    state: State,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,
    pub metadata_cid: Cid,
}

pub enum Msg {
    Metadata(Result<FullPost>),
}

impl Component for Blog {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, metadata_cid } = props;

        let cb = link.callback_once(Msg::Metadata);
        let client = ipfs.clone();

        spawn_local(
            async move { cb.emit(client.dag_get(metadata_cid, Option::<String>::None).await) },
        );

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
            <div class="blog_page">
            <Navbar />
            {
                match &self.state {
                    State::Loading => html! { <Loading /> },
                    State::Ready(md) => self.render_blog(md),
                    State::Error => html! { <Error /> },
                }
            }
            </div>
        }
    }
}

impl Blog {
    fn update_metadata(&mut self, response: Result<FullPost>) -> bool {
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
            <div>
                <div class="post_title"> { &metadata.title } </div>
                <div class="post_image">
                    <img src=format!("ipfs://{}", metadata.image.link.to_string()) alt="This image require IPFS native browser" />
                </div>
                <div class="post_content">
                    <Markdown ipfs=self.ipfs.clone() markdown_cid=metadata.content.link />
                </div>
            </div>
        }
    }
}
