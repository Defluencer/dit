use crate::components::{Navbar, VideoPlayer};
use crate::utils::ipfs::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::video::VideoMetadata;

use cid::Cid;

pub struct Video {
    ipfs: IpfsService,
    metadata: Option<VideoMetadata>,
}

pub enum Msg {
    Metadata(Result<VideoMetadata, ipfs_api::response::Error>),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,
    pub metadata_cid: Cid,
}

impl Component for Video {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, metadata_cid } = props;

        let cb = link.callback(Msg::Metadata);
        let client = ipfs.clone();

        spawn_local(
            async move { cb.emit(ipfs.dag_get(metadata_cid, Option::<String>::None).await) },
        );

        Self {
            ipfs,
            metadata: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata(res) => self.update_metadata(res),
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
                        html! { <VideoPlayer ipfs=self.ipfs.clone() metadata=Some(md.clone()) topic=Option::<String>::None streamer_peer_id=Option::<String>::None /> }
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
    fn update_metadata(
        &mut self,
        response: Result<VideoMetadata, ipfs_api::response::Error>,
    ) -> bool {
        let metadata = match response {
            Ok(md) => md,
            Err(e) => {
                //TODO display error
                // states; loading, video or error
                return false;
            }
        };

        self.metadata = Some(metadata);

        true
    }
}
