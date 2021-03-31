use crate::components::{Navbar, VideoPlayer};
use crate::utils::ipfs::ipfs_dag_get_callback;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::beacon::VideoMetadata;

use cid::Cid;

use ipfs_api::IpfsClient;

pub struct Video {
    metadata: Option<VideoMetadata>,
}

pub enum Msg {
    Metadata((Cid, VideoMetadata)),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub metadata_cid: Cid,
}

impl Component for Video {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ipfs = IpfsClient::default();

        spawn_local(ipfs_dag_get_callback(
            ipfs.clone(),
            props.metadata_cid,
            link.callback(Msg::Metadata),
        ));

        Self { metadata: None }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata((_, metadata)) => self.update_metadata(metadata),
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
                    if let Some(md) = self.metadata.as_ref() {
                        html! { <VideoPlayer metadata=Some(md.clone()) topic=Option::<String>::None streamer_peer_id=Option::<String>::None /> }
                    } else {
                        html! { <div class="center_text"> {"Loading..."} </div> }
                    }
                }
            </div>
        }
    }

    fn destroy(&mut self) {}
}

impl Video {
    fn update_metadata(&mut self, metadata: VideoMetadata) -> bool {
        self.metadata = Some(metadata);

        true
    }
}
