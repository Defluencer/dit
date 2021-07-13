use crate::components::{Navbar, VideoPlayer};
use crate::utils::ipfs::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::video::VideoMetadata;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[allow(clippy::large_enum_variant)]
enum State {
    Loading,
    Ready(VideoMetadata),
    Error(Box<dyn std::error::Error>),
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
            <div class="video_page">
            <Navbar />
            {
                match &self.state {
                    State::Loading => html! { <div class="center_text"> {"Loading..."} </div> },
                    State::Ready(md) => html! { <VideoPlayer ipfs=self.ipfs.clone() metadata=Some(md.clone()) topic=Option::<String>::None streamer_peer_id=Option::<String>::None /> },
                    State::Error(e) => html! { <div class="center_text"> { format!("{:#?}", e) } </div> },
                }
            }
            </div>
        }
    }

    fn destroy(&mut self) {}
}

impl Video {
    fn update_metadata(&mut self, response: Result<VideoMetadata>) -> bool {
        self.state = match response {
            Ok(md) => State::Ready(md),
            Err(e) => State::Error(e),
        };

        true
    }
}
