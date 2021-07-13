use std::collections::HashMap;

use crate::app::ENS_NAME;
use crate::components::{Navbar, VideoThumbnail};
use crate::utils::ipfs::IpfsService;
use crate::utils::local_storage::{get_cid, get_local_storage, set_cid, set_local_beacon};
use crate::utils::web3::Web3Service;

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::Beacon;
use linked_data::feed::Feed;
use linked_data::video::VideoMetadata;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Maintaining an updated content feed should be a different component.
// Specialized component just refer to feed then dag get & deserialize (videos, blog post, etc...).

pub struct Videos {
    link: ComponentLink<Self>,

    ipfs: IpfsService,

    storage: Option<Storage>,

    beacon_cid: Option<Cid>,
    beacon: Option<Beacon>,

    searching: bool,

    list_cid: Option<Cid>,
    feed: Option<Feed>,

    call_count: usize,
    metadata_map: HashMap<Cid, VideoMetadata>,
}

pub enum Msg {
    ResolveName(Result<Cid>),
    Beacon(Result<Beacon>),
    List((Cid, Result<Feed>)),
    ResolveList(Result<(Cid, Feed)>),
    Metadata((Cid, Result<VideoMetadata>)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService, // From app.
    pub web3: Web3Service, // From app.
}

impl Component for Videos {
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
        let name = ENS_NAME.to_owned();

        spawn_local(async move { cb.emit(web3.get_ipfs_content(name).await) });

        Self {
            link,
            ipfs,
            beacon_cid,
            beacon: None,
            searching: true,
            list_cid: None,
            feed: None,
            storage,
            call_count: 0,
            metadata_map: HashMap::with_capacity(10),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ResolveName(result) => self.on_name_resolved(result),
            Msg::Beacon(result) => self.on_beacon_update(result),
            Msg::List((cid, result)) => self.on_feed_update(cid, result),
            Msg::ResolveList(result) => self.on_feed_resolved(result),
            Msg::Metadata((cid, result)) => self.on_video_metadata_update(cid, result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let content = if self.searching {
            html! { <div class="center_text">  {"Loading..."} </div> }
        } else {
            let feed = self.feed.as_ref().unwrap();

            html! {
                <div class="feed">
                {
                    for feed.content.iter().rev().map(|ipld| {
                        let cid = ipld.link;
                        let mt = &self.metadata_map[&cid];
                        html! {
                            <VideoThumbnail metadata_cid=cid metadata=mt />
                        }
                    }
                    )
                }
                </div>
            }
        };

        html! {
            <div class="content_feed_page">
                <Navbar />
                { content }
            </div>
        }
    }
}

impl Videos {
    /// Callback when Ethereum Name Service resolve name to beacon Cid.
    fn on_name_resolved(&mut self, res: Result<Cid>) -> bool {
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

        set_local_beacon(ENS_NAME, &cid, self.storage.as_ref());

        self.beacon_cid = Some(cid);

        false
    }

    /// Callback when IPFS dag get return beacon node.
    fn on_beacon_update(&mut self, res: Result<Beacon>) -> bool {
        let beacon = match res {
            Ok(b) => b,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        if let Some(cid) = get_cid(&beacon.content_feed, self.storage.as_ref()) {
            self.list_cid = Some(cid);

            let cb = self.link.callback_once(Msg::List);
            let client = self.ipfs.clone();

            spawn_local(async move {
                cb.emit((cid, client.dag_get(cid, Option::<String>::None).await))
            });
        }

        let cb = self.link.callback_once(Msg::ResolveList);
        let client = self.ipfs.clone();
        let ipns = beacon.content_feed.clone();

        spawn_local(async move { cb.emit(client.resolve_and_dag_get(ipns).await) });

        self.beacon = Some(beacon);

        false
    }

    /// Callback when IPFS dag get return Feed node.
    fn on_feed_resolved(&mut self, res: Result<(Cid, Feed)>) -> bool {
        let (cid, feed) = match res {
            Ok((cid, feed)) => (cid, feed),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        self.on_feed_update(cid, Ok(feed))
    }

    /// Callback when IPFS resolve and dag get Feed node.
    fn on_feed_update(&mut self, list_cid: Cid, res: Result<Feed>) -> bool {
        let feed = match res {
            Ok(l) => l,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if let Some(old_list_cid) = self.list_cid.as_ref() {
            if *old_list_cid == list_cid && self.feed.is_some() {
                return false;
            }
        }

        let beacon = match self.beacon.as_ref() {
            Some(b) => b,
            None => return false,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Feed Update");

        if let Some(old_list_cid) = self.list_cid.as_ref() {
            if *old_list_cid != list_cid {
                self.list_cid = Some(list_cid);
                set_cid(&beacon.content_feed, &list_cid, self.storage.as_ref());
            }
        } else {
            self.list_cid = Some(list_cid);
            set_cid(&beacon.content_feed, &list_cid, self.storage.as_ref());
        }

        if feed.content.is_empty() {
            self.feed = Some(feed);
            self.searching = false;
            return true;
        }

        for metadata in feed.content.iter().rev() {
            let cb = self.link.callback_once(Msg::Metadata);
            let client = self.ipfs.clone();
            let cid = metadata.link;

            spawn_local(async move {
                cb.emit((cid, client.dag_get(cid, Option::<String>::None).await))
            });
        }

        self.call_count += feed.content.len();

        self.feed = Some(feed);
        self.searching = false;

        false
    }

    /// Callback when IPFS dag get returns VideoMetadata node.
    fn on_video_metadata_update(&mut self, cid: Cid, res: Result<VideoMetadata>) -> bool {
        let metadata = match res {
            Ok(d) => d,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Display Add => {} \n {}",
            &cid.to_string(),
            &serde_json::to_string_pretty(&metadata).expect("Can't print")
        ));

        self.metadata_map.insert(cid, metadata);

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
