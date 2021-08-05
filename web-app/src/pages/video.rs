use std::rc::Rc;

use crate::components::{Error, Loading, Navbar, VideoPlayer};
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::video::VideoMetadata;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum State {
    Loading,
    Ready(Rc<VideoMetadata>),
    Error,
}

pub struct Video {
    ipfs: IpfsService,
    state: State,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,
    pub metadata_cid: Cid,
}

pub enum Msg {
    Metadata(Result<VideoMetadata>),
}

impl Component for Video {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, metadata_cid } = props;

        spawn_local({
            let cb = link.callback_once(Msg::Metadata);
            let ipfs = ipfs.clone();

            async move { cb.emit(ipfs.dag_get(metadata_cid, Option::<String>::None).await) }
        });

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
            <div class="video_page">
            <Navbar />
            {
                match &self.state {
                    State::Loading => html! { <Loading /> },
                    State::Ready(md) => html! { <VideoPlayer ipfs=self.ipfs.clone() metadata=md.clone() /> },
                    State::Error => html! { <Error /> },
                }
            }
            </div>
        }
    }
}

impl Video {
    fn update_metadata(&mut self, response: Result<VideoMetadata>) -> bool {
        self.state = match response {
            Ok(md) => State::Ready(Rc::from(md)),
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                State::Error
            }
        };

        true
    }
}
