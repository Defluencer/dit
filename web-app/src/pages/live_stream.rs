/* use std::str::FromStr;

use crate::components::{ChatWindow, LiveStreamPlayer, Navbar};
use crate::utils::ens::get_beacon_from_name;
use crate::utils::ipfs::ipfs_dag_get_callback;
use crate::utils::local_storage::{get_local_beacon, get_local_storage, set_local_beacon};

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use cid::Cid;

use linked_data::beacon::Beacon;

/// The Live Stream Page
pub struct LiveStream {
    link: ComponentLink<Self>,

    ens_name: String,

    beacon_cid: Option<Cid>,
    beacon: Option<Beacon>,

    storage: Option<Storage>,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ens_name: String,
}

pub enum Msg {
    Name(Cid),
    Beacon((Cid, Beacon)),
}

impl Component for LiveStream {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ens_name = props.ens_name;

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let mut beacon_cid = get_local_beacon(&ens_name, storage.as_ref());

        //Maybe a name or cid
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
            storage,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Name(cid) => self.name_update(cid),
            Msg::Beacon((_, beacon)) => self.beacon_update(beacon),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="live_stream_page">
                <Navbar ens_name=self.ens_name.clone() />
                {
                    if let Some(beacon) = self.beacon.as_ref() {
                        html! {
                            <>
                                <LiveStreamPlayer topic=beacon.topics.live_video.clone() streamer_peer_id=beacon.peer_id.clone() />
                                <ChatWindow topic=beacon.topics.live_chat.clone() />
                            </>
                        }
                    } else {
                        html! {
                            <div class="center_text">  {"Loading..."} </div>
                        }
                    }
                }
            </div>
        }
    }
}

impl LiveStream {
    /// Receive Content hash from ethereum name service then get beacon
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

    /// Receive beacon node
    fn beacon_update(&mut self, beacon: Beacon) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        self.beacon = Some(beacon);

        true
    }
}
 */
