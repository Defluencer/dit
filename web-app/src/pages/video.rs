use crate::components::VideoPlayer;

use crate::utils::{
    get_local_storage, get_local_video_metadata, ipfs_dag_get_metadata, set_local_video_metadata,
};

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::beacon::VideoMetadata;

use cid::Cid;

pub struct Video {
    storage: Option<Storage>,

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
        let window = web_sys::window().expect("Can't get window");

        let storage = get_local_storage(&window);

        let metadata = get_local_video_metadata(&props.metadata_cid, storage.as_ref());

        if metadata.is_none() {
            spawn_local(ipfs_dag_get_metadata(
                props.metadata_cid,
                link.callback(Msg::Metadata),
            ))
        }

        Self { storage, metadata }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Metadata((cid, metadata)) => self.update_metadata(cid, metadata),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if let Some(md) = &self.metadata {
            html! {
                <div class="video_page">
                    <VideoPlayer  metadata=md />
                </div>
            }
        } else {
            html! {
                <div class="video_page">
                    <div class="center_text"> {"Loading..."} </div>
                </div>
            }
        }
    }

    fn destroy(&mut self) {}
}

impl Video {
    fn update_metadata(&mut self, cid: Cid, metadata: VideoMetadata) -> bool {
        set_local_video_metadata(&cid, &metadata, self.storage.as_ref());

        self.metadata = Some(metadata);

        true
    }
}
