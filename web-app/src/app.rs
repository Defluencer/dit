use std::rc::Rc;

use crate::pages::{Blog, ContentFeed, Home, Live, Settings, Video};
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew_router::prelude::{Router, Switch};

use linked_data::beacon::Beacon;
use linked_data::feed::FeedAnchor;
use linked_data::moderation::Bans;
use linked_data::moderation::Moderators;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/#/video/{cid}"]
    Video(Cid),

    #[to = "/#/weblog/{cid}"]
    Blog(Cid),

    #[to = "/#/settings"]
    Settings,

    #[to = "/#/live"]
    Live,

    #[to = "/#/feed"]
    Feed,

    #[to = "/"]
    Home,
}

pub struct App {
    props: Props,
    link: ComponentLink<Self>,

    beacon_cid: Cid,
    beacon: Rc<Beacon>,

    feed_cid: Cid,
    feed: Rc<FeedAnchor>,

    bans_cid: Cid,
    bans: Rc<Bans>,

    mods_cid: Cid,
    mods: Rc<Moderators>,
}

pub enum AppMsg {
    ResolveName(Result<Cid>),
    Beacon(Result<Beacon>),
    Feed(Result<(Cid, FeedAnchor)>),
    BanList(Result<(Cid, Bans)>),
    ModList(Result<(Cid, Moderators)>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub web3: Web3Service,
    pub ipfs: IpfsService,
    pub storage: LocalStorage,
    pub ens_name: Rc<str>,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback_once(AppMsg::ResolveName);
        let client = props.web3.clone();
        let name = props.ens_name.to_string();

        spawn_local(async move { cb.emit(client.get_ipfs_content(name).await) });

        if let Some(cid) = props.storage.get_cid(&props.ens_name) {
            let cb = link.callback_once(AppMsg::Beacon);
            let client = props.ipfs.clone();

            spawn_local(async move { cb.emit(client.dag_get(cid, Option::<String>::None).await) });
        }

        Self {
            props,
            link,

            beacon_cid: Cid::default(),
            beacon: Rc::from(Beacon::default()),

            feed_cid: Cid::default(),
            feed: Rc::from(FeedAnchor::default()),

            bans_cid: Cid::default(),
            bans: Rc::from(Bans::default()),

            mods_cid: Cid::default(),
            mods: Rc::from(Moderators::default()),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AppMsg::ResolveName(result) => self.on_name_resolved(result),
            AppMsg::Beacon(result) => self.on_beacon_update(result),
            AppMsg::Feed(result) => self.on_feed_resolved(result),
            AppMsg::BanList(result) => self.on_ban_list_resolved(result),
            AppMsg::ModList(result) => self.on_mod_list_resolved(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let web3 = self.props.web3.clone();
        let ipfs = self.props.ipfs.clone();
        let storage = self.props.storage.clone();
        let feed = self.feed.clone();
        let beacon = self.beacon.clone();
        let bans = self.bans.clone();
        let mods = self.mods.clone();

        html! {
            <>
                <Router<AppRoute>
                    render = Router::render(move |switch: AppRoute| {
                        match switch {
                            AppRoute::Video(cid) => html! { <Video ipfs=ipfs.clone() metadata_cid=cid /> },
                            AppRoute::Blog(cid) => html! { <Blog ipfs=ipfs.clone() metadata_cid=cid /> },
                            AppRoute::Settings => html! { <Settings storage=storage.clone() /> },
                            AppRoute::Live => html! { <Live ipfs=ipfs.clone() web3=web3.clone() storage=storage.clone() beacon=beacon.clone() bans=bans.clone() mods=mods.clone() /> },
                            AppRoute::Feed => html! { <ContentFeed ipfs=ipfs.clone() storage=storage.clone() feed=feed.clone() /> },
                            AppRoute::Home => html! { <Home /> },
                        }
                    })
                />
            </>
        }
    }
}

impl App {
    /// Callback when Ethereum Name Service resolve name to Beacon CID.
    fn on_name_resolved(&mut self, response: Result<Cid>) -> bool {
        let beacon_cid = match response {
            Ok(cid) => cid,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if self.beacon_cid == beacon_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Cid Update");

        let cb = self.link.callback_once(AppMsg::Beacon);
        let ipfs = self.props.ipfs.clone();

        spawn_local(async move { cb.emit(ipfs.dag_get(beacon_cid, Option::<String>::None).await) });

        self.props
            .storage
            .set_beacon(&self.props.ens_name, &beacon_cid);

        self.beacon_cid = beacon_cid;

        false
    }

    /// Callback when IPFS dag get return beacon node.
    fn on_beacon_update(&mut self, response: Result<Beacon>) -> bool {
        let beacon = match response {
            Ok(beacon) => beacon,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if *self.beacon == beacon {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        let cb = self.link.callback_once(AppMsg::Feed);
        let ipfs = self.props.ipfs.clone();
        let feed = beacon.content_feed.clone();
        spawn_local(async move { cb.emit(ipfs.resolve_and_dag_get(feed).await) });

        if let Some(cid) = self.props.storage.get_cid(&beacon.content_feed) {
            let cb = self.link.callback_once(AppMsg::Feed);
            let ipfs = self.props.ipfs.clone();

            spawn_local(async move {
                match ipfs.dag_get(cid, Option::<&str>::None).await {
                    Ok(node) => cb.emit(Ok((cid, node))),
                    Err(e) => cb.emit(Err(e)),
                }
            });
        }

        let cb = self.link.callback_once(AppMsg::BanList);
        let ipfs = self.props.ipfs.clone();
        let bans = beacon.bans.clone();
        spawn_local(async move { cb.emit(ipfs.resolve_and_dag_get(bans).await) });

        if let Some(cid) = self.props.storage.get_cid(&beacon.bans) {
            let cb = self.link.callback_once(AppMsg::BanList);
            let ipfs = self.props.ipfs.clone();

            spawn_local(async move {
                match ipfs.dag_get(cid, Option::<&str>::None).await {
                    Ok(node) => cb.emit(Ok((cid, node))),
                    Err(e) => cb.emit(Err(e)),
                }
            });
        }

        let cb = self.link.callback_once(AppMsg::ModList);
        let ipfs = self.props.ipfs.clone();
        let mods = beacon.mods.clone();
        spawn_local(async move { cb.emit(ipfs.resolve_and_dag_get(mods).await) });

        if let Some(cid) = self.props.storage.get_cid(&beacon.mods) {
            let cb = self.link.callback_once(AppMsg::ModList);
            let ipfs = self.props.ipfs.clone();

            spawn_local(async move {
                match ipfs.dag_get(cid, Option::<&str>::None).await {
                    Ok(node) => cb.emit(Ok((cid, node))),
                    Err(e) => cb.emit(Err(e)),
                }
            });
        }

        self.beacon = Rc::from(beacon);

        true
    }

    /// Callback when IPFS dag get return Feed node.
    fn on_feed_resolved(&mut self, res: Result<(Cid, FeedAnchor)>) -> bool {
        let (feed_cid, feed) = match res {
            Ok((cid, feed)) => (cid, feed),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if self.feed_cid == feed_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Feed Update");

        self.props
            .storage
            .set_cid(&self.beacon.content_feed, &feed_cid);

        self.feed_cid = feed_cid;
        self.feed = Rc::from(feed);

        true
    }

    /// Callback when IPFS dag get ban list node.
    fn on_ban_list_resolved(&mut self, result: Result<(Cid, Bans)>) -> bool {
        let (bans_cid, bans) = match result {
            Ok((bans_cid, bans)) => (bans_cid, bans),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if self.bans_cid == bans_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Ban List Update");

        self.props.storage.set_cid(&self.beacon.bans, &bans_cid);

        self.bans_cid = bans_cid;
        self.bans = Rc::from(bans);

        true
    }

    /// Callback when IPFS dag get moderator list node.
    fn on_mod_list_resolved(&mut self, result: Result<(Cid, Moderators)>) -> bool {
        let (mods_cid, mods) = match result {
            Ok((mods_cid, mods)) => (mods_cid, mods),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if self.mods_cid == mods_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Moderator List Update");

        self.props.storage.set_cid(&self.beacon.mods, &mods_cid);

        self.mods_cid = mods_cid;
        self.mods = Rc::from(mods);

        true
    }
}
