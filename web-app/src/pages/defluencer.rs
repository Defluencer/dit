use std::collections::HashMap;
use std::str::FromStr;

use crate::components::{ChatWindow, /* LiveStreamPlayer, */ Navbar, VideoThumbnail};
use crate::utils::ens::{get_ens_beacon_async, EthereumNameService};
use crate::utils::ipfs::{ipfs_dag_get_callback, ipfs_resolve_and_get_callback};
use crate::utils::local_storage::{get_cid, get_local_storage, set_cid, set_local_beacon};

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::{Beacon, TempVideoList, VideoList};
use linked_data::video::VideoMetadata;

use cid::Cid;

/// Specific Defluencer Home Page
pub struct Defluencer {
    link: ComponentLink<Self>,

    ens_name: String,

    storage: Option<Storage>,

    searching: bool,

    beacon_cid: Option<Cid>,
    beacon: Option<Beacon>,

    list_cid: Option<Cid>,
    video_list: Option<VideoList>,

    call_count: usize,
    metadata_map: HashMap<Cid, VideoMetadata>,
}

pub enum Msg {
    Name(Result<Cid, ()>),
    Beacon((Cid, Beacon)),
    List((Cid, VideoList)),
    Metadata((Cid, VideoMetadata)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ens_name: String,
}

impl Component for Defluencer {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ens_name = props.ens_name;

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let beacon_cid = match Cid::from_str(&ens_name) {
            Ok(cid) => {
                spawn_local(ipfs_dag_get_callback(cid, link.callback(Msg::Beacon)));

                Some(cid)
            }
            Err(_) => {
                let ens = EthereumNameService::new().expect("Can't get window.ethereum");

                get_ens_beacon_async(ens, ens_name.clone(), link.callback(Msg::Name));

                get_cid(&ens_name, storage.as_ref())
            }
        };

        Self {
            link,
            ens_name,
            searching: true,
            beacon_cid,
            beacon: None,
            list_cid: None,
            video_list: None,
            storage,
            call_count: 0,
            metadata_map: HashMap::with_capacity(10),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Name(result) => self.name_update(result),
            Msg::Beacon((_, beacon)) => self.beacon_update(beacon),
            Msg::List((cid, list)) => self.video_list_update(cid, list),
            Msg::Metadata((cid, metadata)) => self.video_metadata_update(cid, metadata),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if let Some(beacon) = self.beacon.as_ref() {
            if let Some(video_list) = self.video_list.as_ref() {
                return html! {
                    <div class="defluencer_page">
                        <Navbar />
                        <div class="live_stream">
                            //<LiveStreamPlayer topic=beacon.topics.live_video.clone() streamer_peer_id=beacon.peer_id.clone() />
                            <ChatWindow topic=beacon.topics.live_chat.clone() />
                        </div>
                        <div class="video_list">
                        {
                            for video_list.metadata.iter().rev().map(|ipld| {
                                let cid = ipld.link;
                                let mt = &self.metadata_map[&cid];
                                html! {
                                    <VideoThumbnail metadata_cid=cid metadata=mt />
                                }
                            }
                            )
                        }
                        </div>
                    </div>
                };
            }
        }

        if self.searching {
            return html! {
                <div class="defluencer_page">
                    <Navbar />
                    <div class="center_text">  {"Loading..."} </div>
                </div>
            };
        } else {
            return html! {
                <div class="defluencer_page">
                    <Navbar />
                    <div class="center_text">  {"Defluencer not found!"} </div>
                </div>
            };
        }
    }
}

impl Defluencer {
    /// Receive beacon Cid from ENS then get beacon node
    fn name_update(&mut self, result: Result<Cid, ()>) -> bool {
        let cid = match result {
            Ok(cid) => cid,
            Err(_) => {
                self.searching = false;
                return true;
            }
        };

        spawn_local(ipfs_dag_get_callback(cid, self.link.callback(Msg::Beacon)));

        if let Some(beacon_cid) = self.beacon_cid.as_ref() {
            if *beacon_cid == cid {
                return false;
            }
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Name Update");

        set_local_beacon(&self.ens_name, &cid, self.storage.as_ref());

        self.beacon_cid = Some(cid);

        false
    }

    /// Receive beacon node then get video list node
    fn beacon_update(&mut self, beacon: Beacon) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        if let Some(cid) = get_cid(&beacon.video_list, self.storage.as_ref()) {
            self.list_cid = Some(cid);

            spawn_local(ipfs_dag_get_callback(cid, self.link.callback(Msg::List)));
        }

        spawn_local(ipfs_resolve_and_get_callback(
            beacon.video_list.clone(),
            self.link.callback(Msg::List),
        ));

        self.beacon = Some(beacon);

        false
    }

    /// Receive video list, save locally and try to get all metadata
    fn video_list_update(&mut self, list_cid: Cid, list: VideoList) -> bool {
        if let Some(old_list_cid) = self.list_cid.as_ref() {
            if *old_list_cid == list_cid && self.video_list.is_some() {
                return false;
            }
        }

        let beacon = match self.beacon.as_ref() {
            Some(b) => b,
            None => return false,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Video List Update");

        if let Some(old_list_cid) = self.list_cid.as_ref() {
            if *old_list_cid != list_cid {
                self.list_cid = Some(list_cid);
                set_cid(&beacon.video_list, &list_cid, self.storage.as_ref());
            }
        } else {
            self.list_cid = Some(list_cid);
            set_cid(&beacon.video_list, &list_cid, self.storage.as_ref());
        }

        for metadata in list.metadata.iter().rev() {
            spawn_local(ipfs_dag_get_callback(
                metadata.link,
                self.link.callback(Msg::Metadata),
            ))
        }

        self.call_count += list.metadata.len();

        self.video_list = Some(list);

        false
    }

    /// Receive video metadata then update thumbnails
    fn video_metadata_update(&mut self, metadata_cid: Cid, metadata: VideoMetadata) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Display Add => {} \n {}",
            &metadata_cid.to_string(),
            &serde_json::to_string_pretty(&metadata).expect("Can't print")
        ));

        self.metadata_map.insert(metadata_cid, metadata);

        if self.call_count > 0 {
            self.call_count -= 1;
        }

        if self.call_count == 0 {
            #[cfg(debug_assertions)]
            ConsoleService::info("Refresh");

            return true;
        }

        false
    }
}
