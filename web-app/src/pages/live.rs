use std::rc::Rc;

use crate::app::ENS_NAME;
use crate::components::{ChatWindow, Navbar, VideoPlayer};
use crate::utils::ipfs::IpfsService;
use crate::utils::local_storage::{get_cid, get_local_storage, set_local_beacon};
use crate::utils::web3::Web3Service;

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::Beacon;
use linked_data::video::VideoMetadata;

use cid::Cid;

use reqwest::Error;

enum DisplayState {
    Searching,
    Beacon(Beacon),
}

pub struct Live {
    link: ComponentLink<Self>,

    ipfs: IpfsService,
    web3: Web3Service,

    storage: Option<Storage>,

    beacon_cid: Option<Cid>,

    state: DisplayState,
}

pub enum Msg {
    ResolveName(Result<Cid, web3::contract::Error>),
    Beacon(Result<Beacon, Error>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService, // From app.
    pub web3: Web3Service, // From app.
}

impl Component for Live {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, web3 } = props;

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let beacon_cid = get_cid(ENS_NAME, storage.as_ref());

        if let Some(cid) = beacon_cid {
            let cb = link.callback_once(Msg::Beacon);
            let client = ipfs.clone();

            spawn_local(async move { cb.emit(client.dag_get(cid, Option::<String>::None).await) });
        }

        // Check for beacon updates by resolving name.
        let cb = link.callback_once(Msg::ResolveName);
        let client = web3.clone();
        let name = ENS_NAME.to_owned();

        spawn_local(async move { cb.emit(client.get_ipfs_content(name).await) });

        Self {
            link,

            ipfs,
            web3,
            storage,

            beacon_cid,

            state: DisplayState::Searching,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ResolveName(result) => self.on_name_resolved(result),
            Msg::Beacon(result) => self.on_beacon_update(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let content = match &self.state {
            DisplayState::Searching => html! { <div class="center_text">  {"Loading..."} </div> },
            DisplayState::Beacon(beacon) => html! {
                <div class="live_stream">
                    <VideoPlayer ipfs=self.ipfs.clone() metadata=Option::<VideoMetadata>::None topic=Some(beacon.topics.live_video.clone()) streamer_peer_id=Some(beacon.peer_id.clone()) />
                    <ChatWindow ipfs=self.ipfs.clone() web3=self.web3.clone() topic=Rc::from(beacon.topics.live_chat.clone()) ban_list=Rc::from(beacon.bans.clone()) mod_list=Rc::from(beacon.mods.clone())/>
                </div>
            },
        };

        html! {
            <div class="live_page">
                <Navbar />
                { content }
            </div>
        }
    }
}

impl Live {
    /// Callback when Ethereum Name Service resolve name to beacon Cid.
    fn on_name_resolved(&mut self, res: Result<Cid, web3::contract::Error>) -> bool {
        let cid = match res {
            Ok(cid) => cid,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if let Some(beacon_cid) = self.beacon_cid.as_ref() {
            if *beacon_cid == cid {
                return false;
            }
        }

        let cb = self.link.callback_once(Msg::Beacon);
        let client = self.ipfs.clone();

        spawn_local(async move { cb.emit(client.dag_get(cid, Option::<String>::None).await) });

        #[cfg(debug_assertions)]
        ConsoleService::info("Name Update");

        set_local_beacon(&ENS_NAME, &cid, self.storage.as_ref());

        self.beacon_cid = Some(cid);

        false
    }

    /// Callback when IPFS dag get return beacon node.
    fn on_beacon_update(&mut self, res: Result<Beacon, Error>) -> bool {
        let beacon = match res {
            Ok(b) => b,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        self.state = DisplayState::Beacon(beacon);

        true
    }
}
