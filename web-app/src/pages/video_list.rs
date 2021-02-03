use crate::components::{Navbar, VideoThumbnail};
use crate::utils::ipfs::{ipfs_dag_get_callback, ipfs_resolve_and_get_callback};
use crate::utils::local_storage::{get_local_list, get_local_storage, set_local_list};

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::{Beacon, TempVideoList, TempVideoMetadata, VideoList, VideoMetadata};

use cid::Cid;

pub struct VideoOnDemand {
    link: ComponentLink<Self>,

    beacon_cid: Cid,
    beacon: Option<Beacon>,

    list_cid: Option<Cid>,
    video_list: Option<VideoList>,

    storage: Option<Storage>,

    metadata: Vec<(Cid, VideoMetadata)>,
}

pub enum Msg {
    Beacon((Cid, Beacon)),
    List((Cid, VideoList)),
    Metadata((Cid, VideoMetadata)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub beacon_cid: Cid,
}

impl Component for VideoOnDemand {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        spawn_local(ipfs_dag_get_callback(
            props.beacon_cid,
            link.callback(Msg::Beacon),
        ));

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        Self {
            link,
            beacon_cid: props.beacon_cid,
            beacon: None,
            list_cid: None,
            video_list: None,
            storage,
            metadata: Vec::with_capacity(10),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Beacon((_, beacon)) => self.beacon_update(beacon),
            Msg::List((cid, list)) => self.video_list_update(cid, list),
            Msg::Metadata((cid, metadata)) => self.video_metadata_update(cid, metadata),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="vod_page">
            <Navbar beacon_cid=self.beacon_cid />
            {
                if self.metadata.is_empty() {
                    html! {
                        <div class="center_text">  {"Loading..."} </div>
                    }
                } else {
                    html! {
                        {
                            for self.metadata.iter().rev().map(|(cid, mt)| html! {
                                <VideoThumbnail metadata_cid=cid metadata=mt />
                            })
                        }
                    }
                }
            }
            </div>
        }
    }

    /* fn destroy(&mut self) {
        stop_list_update();
    } */
}

impl VideoOnDemand {
    /// Receive beacon node, it then try to get the list
    fn beacon_update(&mut self, beacon: Beacon) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        if let Some(cid) = get_local_list(&beacon.video_list, self.storage.as_ref()) {
            spawn_local(ipfs_dag_get_callback::<TempVideoList, _>(
                cid,
                self.link.callback(Msg::List),
            ));
        }

        spawn_local(ipfs_resolve_and_get_callback::<TempVideoList, _>(
            beacon.video_list.clone(),
            self.link.callback(Msg::List),
        ));

        self.beacon = Some(beacon);

        false
    }

    /// Receive video list, it then save list locally and try to get all metadata
    fn video_list_update(&mut self, new_list_cid: Cid, new_list: VideoList) -> bool {
        let beacon = match self.beacon.as_ref() {
            Some(b) => b,
            None => return false,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Video List Update");

        if self.video_list.is_some() {
            self.metadata.clear();
        }

        self.list_cid = Some(new_list_cid);
        self.video_list = Some(new_list);

        set_local_list(&beacon.video_list, &new_list_cid, self.storage.as_ref());

        for metadata in self.video_list.as_ref().unwrap().metadata.iter() {
            spawn_local(ipfs_dag_get_callback::<TempVideoMetadata, _>(
                metadata.link,
                self.link.callback(Msg::Metadata),
            ))
        }

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
