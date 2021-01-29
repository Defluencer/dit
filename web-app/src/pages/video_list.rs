use crate::components::VideoThumbnail;
use crate::utils::{
    get_local_list, get_local_storage, ipfs_dag_get_callback, ipfs_name_resolve_list,
    /* ipfs_subscribe, ipfs_unsubscribe, */ set_local_list,
};

//use std::convert::TryFrom;

//use wasm_bindgen::prelude::Closure;
//use wasm_bindgen::JsCast;

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::ConsoleService;
//use yew::Callback;

use linked_data::beacon::{TempVideoList, TempVideoMetadata, VideoList, VideoMetadata};
use linked_data::BEACON_IPNS_CID;

use cid::Cid;

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
        let window = web_sys::window().expect("Can't get window");

        let storage = get_local_storage(&window);

        let list_cid = get_local_list(storage.as_ref());

        if let Some(cid) = list_cid {
            let cb = link.callback(Msg::List);

            spawn_local(ipfs_dag_get_callback::<TempVideoList, _>(cid, cb));
        }

        //listen_to_beacon(link.callback(Msg::Beacon));
        spawn_local(ipfs_name_resolve_list(
            BEACON_IPNS_CID,
            link.callback(Msg::Beacon),
        ));

        Self {
            link,
            list_cid,
            video_list: None,
            storage,
            metadata: Vec::with_capacity(10),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Beacon(cid) => self.beacon_update(cid),
            Msg::List((cid, list)) => self.video_list_update(cid, list),
            Msg::Metadata((cid, metadata)) => self.video_metadata_update(cid, metadata),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if self.metadata.is_empty() {
            html! {
                <div class="vod_page">
                    <div class="center_text">  {"Loading..."} </div>
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

    /* fn destroy(&mut self) {
        stop_list_update();
    } */
}

impl VideoOnDemand {
    /// Receive cid of video list, it then try to get the list
    fn beacon_update(&mut self, cid: Cid) -> bool {
        if Some(cid) == self.list_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        let cb = self.link.callback(Msg::List);

        spawn_local(ipfs_dag_get_callback::<TempVideoList, _>(cid, cb));

        false
    }

    /// Receive video list, it then save list locally and try to get all metadata
    fn video_list_update(&mut self, new_list_cid: Cid, new_list: VideoList) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Video List Update");

        let mut new_vids_count = new_list.counter;

        if let Some(old_list) = self.video_list.as_ref() {
            if new_list.counter > old_list.counter {
                new_vids_count = new_list.counter - old_list.counter;
            } else {
                return false;
            }
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("{} New Videos", &new_vids_count));

        set_local_list(&new_list_cid, self.storage.as_ref());

        //iter from newest take only new videos then iter from oldest new videos
        for metadata in new_list.metadata.iter().rev().take(new_vids_count).rev() {
            spawn_local(ipfs_dag_get_callback::<TempVideoMetadata, _>(
                metadata.link,
                self.link.callback(Msg::Metadata),
            ))
        }

        self.list_cid = Some(new_list_cid);
        self.video_list = Some(new_list);

        false
    }

    /// Receive video metadata, it then update thumbnails
    fn video_metadata_update(&mut self, cid: Cid, metadata: VideoMetadata) -> bool {
        let video_list = match self.video_list.as_ref() {
            Some(vl) => vl,
            None => return false,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Video Metadata Update");

        //iter from newest
        for metadata_link in video_list.metadata.iter().rev() {
            let metadata_cid = metadata_link.link;

            if cid == metadata_cid {
                #[cfg(debug_assertions)]
                ConsoleService::info(&format!(
                    "Display Add => {} \n {}",
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
}

/* fn listen_to_beacon(cb: Callback<Cid>) {
    let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Message");

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        if from != STREAMER_PEER_ID {
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

    ipfs_subscribe(CONTENT_UPDATE_TOPIC.into(), pubsub_closure.into_js_value().unchecked_ref());
} */

/* fn stop_list_update() {
    ipfs_unsubscribe(CONTENT_UPDATE_TOPIC.into());
} */
