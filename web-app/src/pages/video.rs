use crate::components::VideoPlayer;

use crate::utils::ipfs_dag_get_metadata;

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

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
        let storage = get_local_storage();

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
                    {"Loading..."}
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

//TODO de-duplicate function in video_list

fn get_local_storage() -> Option<Storage> {
    #[cfg(debug_assertions)]
    ConsoleService::info("Get Local Storage");

    let window = web_sys::window().expect("Can't get window");

    match window.local_storage() {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));

            None
        }
    }
}

fn get_local_video_metadata(cid: &Cid, storage: Option<&Storage>) -> Option<VideoMetadata> {
    let storage = match storage {
        Some(st) => st,
        None => return None,
    };

    let item = match storage.get_item(&cid.to_string()) {
        Ok(option) => option,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    };

    let item = item?;

    let metadata = match serde_json::from_str(&item) {
        Ok(md) => md,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return None;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Get => {} \n {}",
        &cid.to_string(),
        &serde_json::to_string_pretty(&metadata).expect("Can't print")
    ));

    Some(metadata)
}

fn set_local_video_metadata(cid: &Cid, metadata: &VideoMetadata, storage: Option<&Storage>) {
    let storage = match storage {
        Some(st) => st,
        None => return,
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Storage Set => {} \n {}",
        &cid.to_string(),
        &serde_json::to_string_pretty(&metadata).expect("Can't print")
    ));

    let item = match serde_json::to_string(metadata) {
        Ok(s) => s,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    if let Err(e) = storage.set_item(&cid.to_string(), &item) {
        ConsoleService::error(&format!("{:?}", e));
    }
}
