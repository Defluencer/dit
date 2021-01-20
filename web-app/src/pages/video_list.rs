use crate::components::VideoThumbnail;
use crate::utils::{ipfs_dag_get_list, ipfs_dag_get_metadata, ipfs_subscribe, ipfs_unsubscribe};

use std::convert::TryFrom;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::beacon::{VideoList, VideoMetadata};

use cid::Cid;

//Hard-coded for now
const TOPIC: &str = "videoupdate";
const INFLUENCER_PEER_ID: &str = "12D3KooWATLaZPouZ8DDXjsxuLsMv61CHFCN8y4Ho4iG182uMa4E";

const VIDEO_LIST_LOCAL_KEY: &str = "video_list";

pub struct VideoOnDemand {
    link: ComponentLink<Self>,

    list_cid: Option<Cid>,
    video_list: Option<VideoList>,

    storage: Option<Storage>,

    metadata: Vec<(Cid, VideoMetadata)>,
}

pub enum Msg {
    Beacon(Cid),
    List((Cid, VideoList)),
    Metadata((Cid, VideoMetadata)),
}

impl Component for VideoOnDemand {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        listen_to_update(link.callback(Msg::Beacon));

        let mut vod = Self {
            link,
            list_cid: None,
            video_list: None,
            storage: None,
            metadata: Vec::with_capacity(10),
        };

        vod.get_local_storage();

        vod.get_videos_metadata();

        vod
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Beacon(cid) => self.update_cid(cid),
            Msg::List((cid, list)) => self.update_list(cid, list),
            Msg::Metadata((cid, metadata)) => self.update_metadata(cid, metadata),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if self.metadata.is_empty() {
            html! {
                <div class="vod_page">
                    <div> {"Loading..."} </div>
                </div>
            }
        } else {
            html! {
                <div class="vod_page">
                {
                    for self.metadata.iter().rev().map(|(cid, mt)| html! {
                        <VideoThumbnail metadata_cid=cid metadata=mt />
                    })
                }
                </div>
            }
        }
    }

    fn destroy(&mut self) {
        stop_list_update();
    }
}

impl VideoOnDemand {
    fn get_local_storage(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Get Local Storage");

        let window = web_sys::window().expect("Can't get window");

        self.storage = match window.local_storage() {
            Ok(option) => option,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));

                None
            }
        }
    }

    fn get_videos_metadata(&mut self) {
        let list = match self.get_local_list() {
            Some(vl) => vl,
            None => return,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Get Videos Metadata");

        //iter from oldest
        for cid in list.metadata.iter() {
            match self.get_local_video_metadata(&cid.link) {
                Some(video_md) => self.metadata.push((cid.link, video_md)),
                None => spawn_local(ipfs_dag_get_metadata(
                    cid.link,
                    self.link.callback(Msg::Metadata),
                )),
            }
        }
    }

    fn get_local_list(&self) -> Option<VideoList> {
        let storage = match &self.storage {
            Some(st) => st,
            None => return None,
        };

        let item = match storage.get_item(VIDEO_LIST_LOCAL_KEY) {
            Ok(option) => option,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return None;
            }
        };

        let item = item?;

        let list = match serde_json::from_str(&item) {
            Ok(list) => list,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return None;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Storage Get => {} \n {}",
            VIDEO_LIST_LOCAL_KEY,
            &serde_json::to_string_pretty(&list).expect("Can't print")
        ));

        Some(list)
    }

    fn get_local_video_metadata(&self, cid: &Cid) -> Option<VideoMetadata> {
        let storage = match &self.storage {
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
            Ok(list) => list,
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

    /// Called when a beacon msg is received then ipfs dag get the video list if needed
    fn update_cid(&mut self, cid: Cid) -> bool {
        if Some(cid) == self.list_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Update Cid");

        //TODO deserialize to beacon msg instead of list

        let cb = self.link.callback(Msg::List);

        spawn_local(ipfs_dag_get_list(cid, cb));

        false
    }

    /// Called when ipfs dag get returns then save list locally, get all videos metadata
    fn update_list(&mut self, new_list_cid: Cid, new_list: VideoList) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Update List");

        let mut new_vids = new_list.counter;

        if let Some(old_list) = self.video_list.as_ref() {
            if new_list.counter > old_list.counter {
                new_vids = new_list.counter - old_list.counter;
            } else {
                return false;
            }
        }

        self.save_local_list(&new_list);

        let mut update = false;

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("{} New Videos", &new_vids));

        //iter from newest take only new video then iter from oldest new video
        for metadata in new_list.metadata.iter().rev().take(new_vids).rev() {
            let cid = metadata.link;

            if let Some(metadata) = self.get_local_video_metadata(&cid) {
                #[cfg(debug_assertions)]
                ConsoleService::info(&format!(
                    "Add Display => {} \n {}",
                    &cid.to_string(),
                    &serde_json::to_string_pretty(&metadata).expect("Can't print")
                ));

                self.metadata.push((cid, metadata));

                update = true;

                continue;
            }

            spawn_local(ipfs_dag_get_metadata(
                cid,
                self.link.callback(Msg::Metadata),
            ))
        }

        self.list_cid = Some(new_list_cid);
        self.video_list = Some(new_list);

        update
    }

    fn save_local_list(&self, list: &VideoList) {
        let storage = match &self.storage {
            Some(st) => st,
            None => return,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Storage Set => {} \n {}",
            VIDEO_LIST_LOCAL_KEY,
            &serde_json::to_string_pretty(&list).expect("Can't print")
        ));

        let item = match serde_json::to_string(list) {
            Ok(s) => s,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        if let Err(e) = storage.set_item(VIDEO_LIST_LOCAL_KEY, &item) {
            ConsoleService::error(&format!("{:?}", e));
        }
    }

    /// Called when ipfs dag get returns video metadata then update local storage and thumbnails
    fn update_metadata(&mut self, cid: Cid, metadata: VideoMetadata) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Update Metadata");

        let video_list = match self.video_list.as_ref() {
            Some(vl) => vl,
            None => return false,
        };

        for video_list_cid in video_list.metadata.iter().rev() {
            let video_list_cid = video_list_cid.link;

            if cid == video_list_cid {
                self.set_local_video_metadata(&cid, &metadata);

                #[cfg(debug_assertions)]
                ConsoleService::info(&format!(
                    "Add Display => {} \n {}",
                    &cid.to_string(),
                    &serde_json::to_string_pretty(&metadata).expect("Can't print")
                ));

                self.metadata.push((cid, metadata));

                if self.metadata.len() == video_list.metadata.len() {
                    #[cfg(debug_assertions)]
                    ConsoleService::info("Refresh");

                    return true;
                }

                return false;
            }
        }

        false
    }

    fn set_local_video_metadata(&self, cid: &Cid, metadata: &VideoMetadata) {
        let storage = match &self.storage {
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
}

fn listen_to_update(cb: Callback<Cid>) {
    let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Message");

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        if from != INFLUENCER_PEER_ID {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Unauthorized Sender");
            return;
        }

        let data_utf8 = match String::from_utf8(data) {
            Ok(string) => string,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::warn("Invalid UTF-8");
                return;
            }
        };

        let cid = match Cid::try_from(data_utf8) {
            Ok(cid) => cid,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::warn("Invalid CID");
                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Message => {}", cid.to_string()));

        cb.emit(cid);
    }) as Box<dyn Fn(String, Vec<u8>)>);

    ipfs_subscribe(TOPIC.into(), pubsub_closure.into_js_value().unchecked_ref());
}

fn stop_list_update() {
    ipfs_unsubscribe(TOPIC.into());
}
