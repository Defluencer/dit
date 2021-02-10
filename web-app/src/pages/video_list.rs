use std::collections::HashSet;
use std::str::FromStr;

use crate::components::{Navbar, VideoThumbnail};
use crate::utils::ens::get_beacon_from_name;
use crate::utils::ipfs::{ipfs_dag_get_callback, ipfs_resolve_and_get_callback};
use crate::utils::local_storage::{
    get_local_beacon, get_local_list, get_local_storage, set_local_beacon, set_local_list,
};

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::{Beacon, TempVideoList, TempVideoMetadata, VideoList, VideoMetadata};

use cid::Cid;

/// The Video on Demand Page
pub struct VideoOnDemand {
    link: ComponentLink<Self>,

    ens_name: String,

    beacon_cid: Option<Cid>,
    beacon: Option<Beacon>,

    list_cid: Option<Cid>,
    video_list: Option<VideoList>,

    storage: Option<Storage>,

    metadata_set: HashSet<Cid>,
    metadata_vec: Vec<(Cid, VideoMetadata)>,
}

pub enum Msg {
    Name(Cid),
    Beacon((Cid, Beacon)),
    List((Cid, VideoList)),
    Metadata((Cid, VideoMetadata)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ens_name: String,
}

impl Component for VideoOnDemand {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ens_name = props.ens_name;

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let mut beacon_cid = get_local_beacon(&ens_name, storage.as_ref());

        //May be a name or cid
        if let Ok(cid) = Cid::from_str(&ens_name) {
            beacon_cid = Some(cid);
        } else {
            spawn_local(get_beacon_from_name(
                ens_name.clone(),
                link.callback(Msg::Name),
            ));
        }

        if let Some(cid) = beacon_cid {
            spawn_local(ipfs_dag_get_callback(cid, link.callback(Msg::Beacon)));
        }

        Self {
            link,
            ens_name,
            beacon_cid,
            beacon: None,
            list_cid: None,
            video_list: None,
            storage,
            metadata_set: HashSet::with_capacity(10),
            metadata_vec: Vec::with_capacity(10),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Name(cid) => self.name_update(cid),
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
                <Navbar ens_name=self.ens_name.clone() />
                {
                    if self.metadata_vec.is_empty() {
                        html! {
                            <div class="center_text">  {"Loading..."} </div>
                        }
                    } else {
                        html! {
                            {
                                for self.metadata_vec.iter().rev().map(|(cid, mt)| html! {
                                    <VideoThumbnail metadata_cid=cid metadata=mt />
                                })
                            }
                        }
                    }
                }
            </div>
        }
    }
}

impl VideoOnDemand {
    /// Receive beacon Cid from ENS then get beacon node
    fn name_update(&mut self, cid: Cid) -> bool {
        if let Some(beacon_cid) = self.beacon_cid.as_ref() {
            if *beacon_cid == cid {
                return false;
            }
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Name Update");

        set_local_beacon(&self.ens_name, &cid, self.storage.as_ref());

        spawn_local(ipfs_dag_get_callback(cid, self.link.callback(Msg::Beacon)));

        self.beacon_cid = Some(cid);

        false
    }

    /// Receive beacon node then get video list node
    fn beacon_update(&mut self, beacon: Beacon) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        if let Some(cid) = get_local_list(&beacon.video_list, self.storage.as_ref()) {
            self.list_cid = Some(cid);

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

    /// Receive video list, save locally and try to get all metadata
    fn video_list_update(&mut self, cid: Cid, list: VideoList) -> bool {
        if let Some(old_list_cid) = self.list_cid.as_ref() {
            if *old_list_cid == cid && self.video_list.is_some() {
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
            if *old_list_cid != cid {
                self.list_cid = Some(cid);
                set_local_list(&beacon.video_list, &cid, self.storage.as_ref());
            }
        } else {
            self.list_cid = Some(cid);
            set_local_list(&beacon.video_list, &cid, self.storage.as_ref());
        }

        self.video_list = Some(list);

        self.metadata_set.clear();
        self.metadata_vec.clear();

        for metadata in self.video_list.as_ref().unwrap().metadata.iter() {
            spawn_local(ipfs_dag_get_callback::<TempVideoMetadata, _>(
                metadata.link,
                self.link.callback(Msg::Metadata),
            ))
        }

        false
    }

    /// Receive video metadata then update thumbnails
    fn video_metadata_update(&mut self, cid: Cid, metadata: VideoMetadata) -> bool {
        let video_list = match self.video_list.as_ref() {
            Some(vl) => vl,
            None => return false,
        };

        //iter from newest
        for metadata_link in video_list.metadata.iter().rev() {
            let metadata_cid = metadata_link.link;

            if cid == metadata_cid {
                if self.metadata_set.insert(cid) {
                    #[cfg(debug_assertions)]
                    ConsoleService::info(&format!(
                        "Display Add => {} \n {}",
                        &cid.to_string(),
                        &serde_json::to_string_pretty(&metadata).expect("Can't print")
                    ));

                    self.metadata_vec.push((cid, metadata));
                }

                if self.metadata_vec.len() == video_list.metadata.len() {
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
