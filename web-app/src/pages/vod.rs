use crate::components::VideoPlayer;
use crate::utils::{ipfs_dag_get, ipfs_subscribe, ipfs_unsubscribe};

use std::convert::TryFrom;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::beacon::VideoList;

use cid::Cid;

const TOPIC: &str = "videoupdatetopic";
const INFLUENCER_PEER_ID: &str = "12D3KooWAPZ3QZnZUJw3BgEX9F7XL383xFNiKQ5YKANiRC3NWvpo";

pub struct VideoOnDemand {
    link: ComponentLink<Self>,

    video_list: Option<VideoList>,

    list_cid: Option<Cid>,
}

pub enum Msg {
    Beacon(Cid),
    Dag((Cid, VideoList)),
}

impl Component for VideoOnDemand {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        listen_video_list_update(link.callback(Msg::Beacon));

        let window = web_sys::window().expect("Can't get window");

        let storage = match window.local_storage() {
            Ok(option) => option,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return Self {
                    link,
                    video_list: None,
                    list_cid: None,
                };
            }
        };

        let storage = match storage {
            Some(db) => db,
            None => {
                return Self {
                    link,
                    video_list: None,
                    list_cid: None,
                };
            }
        };

        let item = match storage.get_item("videos") {
            Ok(option) => option,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return Self {
                    link,
                    video_list: None,
                    list_cid: None,
                };
            }
        };

        let item = match item {
            Some(json) => json,
            None => {
                return Self {
                    link,
                    video_list: None,
                    list_cid: None,
                };
            }
        };

        let list = match serde_json::from_str(&item) {
            Ok(list) => list,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return Self {
                    link,
                    video_list: None,
                    list_cid: None,
                };
            }
        };

        Self {
            link,
            video_list: Some(list),
            list_cid: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Beacon(cid) => {
                if self.list_cid == Some(cid) {
                    return false;
                }

                let cb = self.link.callback(Msg::Dag);

                spawn_local(get_video_list_async(cid, cb));

                false
            }
            Msg::Dag((cid, list)) => {
                if self.video_list.is_none() {
                    self.list_cid = Some(cid);
                    self.video_list = Some(list);

                    return true;
                }

                if list.counter > self.video_list.as_ref().unwrap().counter {
                    self.list_cid = Some(cid);
                    self.video_list = Some(list);

                    return true;
                }

                //TODO if local store has more up to date beacon msg then the one received send it

                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if self.video_list.is_none() {
            html! {
                <div class="vod_page">
                    {"Loading..."}
                </div>
            }
        } else {
            let list = &self.video_list.as_ref().unwrap().metadata;

            html! {
                <div class="vod_page">
                {
                    for list.iter().map(|md| html! {
                        <VideoPlayer metadata=md />
                        //TODO add clickable box that launch VideoPlayer
                    })
                }
                </div>
            }
        }
    }

    fn destroy(&mut self) {
        stop_video_list_update();
    }
}

async fn get_video_list_async(cid: Cid, cb: Callback<(Cid, VideoList)>) {
    let video_list = match ipfs_dag_get(&cid.to_string()).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let video_list: VideoList = match video_list.into_serde() {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    cb.emit((cid, video_list));
}

fn listen_video_list_update(cb: Callback<Cid>) {
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

fn stop_video_list_update() {
    ipfs_unsubscribe(TOPIC.into());
}
