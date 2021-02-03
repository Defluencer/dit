use crate::components::{ChatWindow, LiveStreamPlayer, Navbar};
use crate::utils::ipfs::ipfs_dag_get_callback;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use cid::Cid;

use linked_data::beacon::Beacon;

pub struct LiveStream {
    beacon_cid: Cid,
    beacon: Option<Beacon>,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub beacon_cid: Cid,
}

pub enum Msg {
    Beacon((Cid, Beacon)),
}

impl Component for LiveStream {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        spawn_local(ipfs_dag_get_callback(
            props.beacon_cid,
            link.callback(Msg::Beacon),
        ));

        Self {
            beacon_cid: props.beacon_cid,
            beacon: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Beacon((_, beacon)) => self.beacon_update(beacon),
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="live_stream_page">
                <Navbar beacon_cid=self.beacon_cid />
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
    /// Receive cid of beacon node
    fn beacon_update(&mut self, beacon: Beacon) -> bool {
        self.beacon = Some(beacon);

        true
    }
}
