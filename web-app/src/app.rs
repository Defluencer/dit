use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::rc::Rc;

use crate::pages::{Content, ContentFeed, Home, Live, Settings};
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;
use yew_router::prelude::{Router, Switch};

use linked_data::beacon::Beacon;
use linked_data::comments::Commentary;
use linked_data::feed::{ContentCache, FeedAnchor};
use linked_data::friends::Friendlies;
use linked_data::moderation::Bans;
use linked_data::moderation::Moderators;

use either::Either;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type FeedCallback = Callback<(String, Cid, Result<(Cid, FeedAnchor)>)>;
type CommentCallback = Callback<(String, Cid, Result<(Cid, Commentary)>)>;

#[derive(Switch, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[to = "/#/content/{cid}"]
    Content(Cid),

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

    peer_id: Rc<Option<String>>,
    peer_id_cb: Callback<Result<String>>,

    name_cb: Callback<(String, Result<Cid>)>,

    beacon_set: HashSet<Cid>,
    beacon: Option<Rc<Beacon>>,
    beacon_cb: Callback<(Cid, Result<Beacon>)>,

    /// Maps IPNS to FeedAnchors
    feed_set: HashMap<Cid, Cid>,
    feed_cb: FeedCallback,

    content: Rc<ContentCache>,

    /// Maps IPNS to Commentary
    comments_set: HashMap<Cid, Cid>,
    comments_cb: CommentCallback,

    bans_cid: Option<Cid>,
    bans: Rc<Bans>,
    bans_cb: Callback<(Cid, Result<(Cid, Bans)>)>,

    mods_cid: Option<Cid>,
    mods: Rc<Moderators>,
    mods_cb: Callback<(Cid, Result<(Cid, Moderators)>)>,

    friends_cid: Option<Cid>,
    friends: Rc<Friendlies>,
    friends_cb: Callback<(Cid, Result<(Cid, Friendlies)>)>,
}

#[allow(clippy::large_enum_variant)]
pub enum AppMsg {
    PeerID(Result<String>),
    ENSResolve((String, Result<Cid>)),
    Beacon((Cid, Result<Beacon>)),
    Feed((String, Cid, Result<(Cid, FeedAnchor)>)),
    Bans((Cid, Result<(Cid, Bans)>)),
    Mods((Cid, Result<(Cid, Moderators)>)),
    Comments((String, Cid, Result<(Cid, Commentary)>)),
    Friends((Cid, Result<(Cid, Friendlies)>)),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub web3: Web3Service,
    pub ipfs: IpfsService,
    pub storage: LocalStorage,
    pub beacon: &'static str,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let app = Self {
            props,

            peer_id: Rc::from(None),
            peer_id_cb: link.callback(AppMsg::PeerID),

            name_cb: link.callback(AppMsg::ENSResolve),

            beacon_set: HashSet::with_capacity(10),
            beacon: None,
            beacon_cb: link.callback(AppMsg::Beacon),

            feed_set: HashMap::with_capacity(10),
            feed_cb: link.callback(AppMsg::Feed),

            content: Rc::from(ContentCache::create()),

            comments_set: HashMap::with_capacity(10),
            comments_cb: link.callback(AppMsg::Comments),

            bans_cid: None,
            bans: Rc::from(Bans::default()),
            bans_cb: link.callback(AppMsg::Bans),

            mods_cid: None,
            mods: Rc::from(Moderators::default()),
            mods_cb: link.callback(AppMsg::Mods),

            friends_cid: None,
            friends: Rc::from(Friendlies::default()),
            friends_cb: link.callback(AppMsg::Friends),
        };

        app.check_ipfs();
        app.get_beacon(&app.props.beacon);

        app
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AppMsg::PeerID(result) => self.on_peer_id(result),
            AppMsg::ENSResolve(result) => self.on_name(result),
            AppMsg::Beacon(result) => self.on_beacon(result),
            AppMsg::Feed(result) => self.on_feed(result),
            AppMsg::Bans(result) => self.on_ban_list(result),
            AppMsg::Mods(result) => self.on_mod_list(result),
            AppMsg::Comments(result) => self.on_comments(result),
            AppMsg::Friends(result) => self.on_friends(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let peer_id = self.peer_id.clone();
        let web3 = self.props.web3.clone();
        let ipfs = self.props.ipfs.clone();
        let storage = self.props.storage.clone();
        let content = self.content.clone();
        let beacon = self.beacon.clone().unwrap_or_default();
        let bans = self.bans.clone();
        let mods = self.mods.clone();
        //let friends = self.friends.clone();

        html! {
            <>
                <Router<AppRoute>
                    render = Router::render(move |switch: AppRoute| {
                        match switch {
                            AppRoute::Content(cid) => html! { <Content ipfs=ipfs.clone() cid=cid content=content.clone() /> },
                            AppRoute::Settings => html! { <Settings storage=storage.clone() peer_id=peer_id.clone() /> },
                            AppRoute::Live => html! { <Live peer_id=peer_id.clone() ipfs=ipfs.clone() web3=web3.clone() storage=storage.clone() beacon=beacon.clone() bans=bans.clone() mods=mods.clone() /> },
                            AppRoute::Feed => html! { <ContentFeed ipfs=ipfs.clone() storage=storage.clone() content=content.clone() peer_id=peer_id.clone() /> },
                            AppRoute::Home => html! { <Home /> },
                        }
                    })
                />
            </>
        }
    }
}

impl App {
    fn check_ipfs(&self) {
        spawn_local({
            let cb = self.peer_id_cb.clone();
            let ipfs = self.props.ipfs.clone();

            async move { cb.emit(ipfs.ipfs_node_id().await) }
        });
    }

    fn on_peer_id(&mut self, response: Result<String>) -> bool {
        let id = match response {
            Ok(id) => id,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        self.peer_id = Rc::from(Some(id));

        true
    }

    /// Resolve ENS name and check local storage for a beacon.
    fn get_beacon(&self, beacon: &str) {
        if let Ok(cid) = Cid::try_from(beacon) {
            spawn_local({
                let beacon_cb = self.beacon_cb.clone();
                let ipfs = self.props.ipfs.clone();

                async move { beacon_cb.emit((cid, ipfs.dag_get(cid, Option::<&str>::None).await)) }
            });

            return;
        };

        spawn_local({
            let cb = self.name_cb.clone();
            let web3 = self.props.web3.clone();
            let name = beacon.to_owned();

            async move { cb.emit((name.clone(), web3.get_ipfs_content(name).await)) }
        });

        if let Some(cid) = self.props.storage.get_cid(beacon) {
            spawn_local({
                let cb = self.beacon_cb.clone();
                let ipfs = self.props.ipfs.clone();

                async move { cb.emit((cid, ipfs.dag_get(cid, Option::<String>::None).await)) }
            });
        }
    }

    /// Callback when Ethereum Name Service resolve any name.
    fn on_name(&mut self, res: (String, Result<Cid>)) -> bool {
        let (name, beacon_cid) = match res {
            (name, Ok(cid)) => (name, cid),
            (_, Err(e)) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if self.beacon_set.contains(&beacon_cid) {
            return false;
        }

        spawn_local({
            let cb = self.beacon_cb.clone();
            let ipfs = self.props.ipfs.clone();

            async move {
                cb.emit((
                    beacon_cid,
                    ipfs.dag_get(beacon_cid, Option::<String>::None).await,
                ))
            }
        });

        self.props.storage.set_cid(&name, &beacon_cid);

        #[cfg(debug_assertions)]
        ConsoleService::info("App ENS Name Resolved");

        false
    }

    /// Callback when IPFS dag get return any beacon.
    fn on_beacon(&mut self, response: (Cid, Result<Beacon>)) -> bool {
        let (beacon_cid, beacon) = match response {
            (beacon_cid, Ok(res)) => (beacon_cid, res),
            (_, Err(e)) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if !self.beacon_set.insert(beacon_cid) {
            return false;
        }

        spawn_local({
            let cb = self.feed_cb.clone();
            let ipfs = self.props.ipfs.clone();
            let feed = beacon.content_feed;
            let name = beacon.display_name.clone();

            async move { cb.emit((name, feed, ipfs.resolve_and_dag_get(feed).await)) }
        });

        if let Some(cid) = self.props.storage.get_cid(&beacon.content_feed.to_string()) {
            spawn_local({
                let cb = self.feed_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let feed = beacon.content_feed;
                let name = beacon.display_name.clone();

                async move {
                    match ipfs.dag_get(cid, Option::<&str>::None).await {
                        Ok(node) => cb.emit((name, feed, Ok((cid, node)))),
                        Err(e) => cb.emit((name, feed, Err(e))),
                    }
                }
            });
        }

        if let Some(comments) = beacon.comments {
            spawn_local({
                let cb = self.comments_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let name = beacon.display_name.clone();

                async move { cb.emit((name, comments, ipfs.resolve_and_dag_get(comments).await)) }
            });

            if let Some(cid) = self.props.storage.get_cid(&comments.to_string()) {
                spawn_local({
                    let cb = self.comments_cb.clone();
                    let ipfs = self.props.ipfs.clone();
                    let name = beacon.display_name.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit((name, comments, Ok((cid, node)))),
                            Err(e) => cb.emit((name, comments, Err(e))),
                        }
                    }
                });
            }
        }

        if self.beacon.is_some() {
            //Only resolve the feed and comments of your friends.
            return false;
        }

        if let Some(friends) = beacon.friends {
            spawn_local({
                let cb = self.friends_cb.clone();
                let ipfs = self.props.ipfs.clone();

                async move { cb.emit((friends, ipfs.resolve_and_dag_get(friends).await)) }
            });

            if let Some(cid) = self.props.storage.get_cid(&friends.to_string()) {
                spawn_local({
                    let cb = self.friends_cb.clone();
                    let ipfs = self.props.ipfs.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit((friends, Ok((cid, node)))),
                            Err(e) => cb.emit((friends, Err(e))),
                        }
                    }
                });
            }
        }

        if let Some(bans) = beacon.bans {
            spawn_local({
                let cb = self.bans_cb.clone();
                let ipfs = self.props.ipfs.clone();

                async move { cb.emit((bans, ipfs.resolve_and_dag_get(bans).await)) }
            });

            if let Some(cid) = self.props.storage.get_cid(&bans.to_string()) {
                spawn_local({
                    let cb = self.bans_cb.clone();
                    let ipfs = self.props.ipfs.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit((bans, Ok((cid, node)))),
                            Err(e) => cb.emit((bans, Err(e))),
                        }
                    }
                });
            }
        }

        if let Some(mods) = beacon.mods {
            spawn_local({
                let cb = self.mods_cb.clone();
                let ipfs = self.props.ipfs.clone();

                async move { cb.emit((mods, ipfs.resolve_and_dag_get(mods).await)) }
            });

            if let Some(cid) = self.props.storage.get_cid(&mods.to_string()) {
                spawn_local({
                    let cb = self.mods_cb.clone();
                    let ipfs = self.props.ipfs.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit((mods, Ok((cid, node)))),
                            Err(e) => cb.emit((mods, Err(e))),
                        }
                    }
                });
            }
        }

        if self.beacon.is_none() {
            self.beacon = Rc::from(beacon).into();
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("App Beacon Updated");

        false
    }

    /// Callback when IPFS dag get return any content feed.
    fn on_feed(&mut self, res: (String, Cid, Result<(Cid, FeedAnchor)>)) -> bool {
        let (name, ipns, feed_cid, feed) = match res {
            (name, ipns, Ok((cid, feed))) => (name, ipns, cid, feed),
            (_, _, Err(e)) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if Some(feed_cid) == self.feed_set.insert(ipns, feed_cid) {
            return false;
        }

        Rc::make_mut(&mut self.content).insert_content(name, feed);

        self.props.storage.set_cid(&ipns.to_string(), &feed_cid);

        #[cfg(debug_assertions)]
        ConsoleService::info("App Content Feed Updated");

        true
    }

    /// Callback when IPFS dag get return any comments.
    fn on_comments(&mut self, res: (String, Cid, Result<(Cid, Commentary)>)) -> bool {
        let (name, ipns, comments_cid, comments) = match res {
            (name, ipns, Ok((cid, comments))) => (name, ipns, cid, comments),
            (_, _, Err(e)) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if Some(comments_cid) == self.comments_set.insert(ipns, comments_cid) {
            return false;
        }

        Rc::make_mut(&mut self.content).insert_comments(name, comments);

        self.props.storage.set_cid(&ipns.to_string(), &comments_cid);

        #[cfg(debug_assertions)]
        ConsoleService::info("App Comment List Updated");

        true
    }

    /// Callback when IPFS dag get return your ban list.
    fn on_ban_list(&mut self, res: (Cid, Result<(Cid, Bans)>)) -> bool {
        let (ipns, bans_cid, bans) = match on_node(res) {
            Some(res) => res,
            None => return false,
        };

        if Some(bans_cid) == self.bans_cid {
            return false;
        }

        self.props.storage.set_cid(&ipns.to_string(), &bans_cid);

        self.bans_cid = bans_cid.into();
        self.bans = Rc::from(bans);

        #[cfg(debug_assertions)]
        ConsoleService::info("App Ban List Updated");

        true
    }

    /// Callback when IPFS dag get return your moderators.
    fn on_mod_list(&mut self, res: (Cid, Result<(Cid, Moderators)>)) -> bool {
        let (ipns, mods_cid, mods) = match on_node(res) {
            Some(res) => res,
            None => return false,
        };

        if Some(mods_cid) == self.mods_cid {
            return false;
        }

        self.props.storage.set_cid(&ipns.to_string(), &mods_cid);

        self.mods_cid = mods_cid.into();
        self.mods = Rc::from(mods);

        #[cfg(debug_assertions)]
        ConsoleService::info("App Moderator List Updated");

        true
    }

    /// Callback when IPFS dag get return your friend list.
    fn on_friends(&mut self, res: (Cid, Result<(Cid, Friendlies)>)) -> bool {
        let (ipns, friends_cid, friends) = match on_node(res) {
            Some(res) => res,
            None => return false,
        };

        if Some(friends_cid) == self.friends_cid {
            return false;
        }

        for friend in friends.friends.iter() {
            match &friend.friend {
                Either::Right(ipld) => {
                    spawn_local({
                        let beacon_cb = self.beacon_cb.clone();
                        let cid = ipld.link;
                        let ipfs = self.props.ipfs.clone();

                        async move {
                            beacon_cb.emit((cid, ipfs.dag_get(cid, Option::<&str>::None).await))
                        }
                    });
                }
                Either::Left(name) => {
                    self.get_beacon(name);
                }
            }
        }

        self.props.storage.set_cid(&ipns.to_string(), &friends_cid);

        self.friends_cid = friends_cid.into();
        self.friends = Rc::from(friends);

        #[cfg(debug_assertions)]
        ConsoleService::info("App Friend List Updated");

        true
    }
}

fn on_node<T>(res: (Cid, Result<(Cid, T)>)) -> Option<(Cid, Cid, T)> {
    match res {
        (ipns, Ok((cid, node))) => Some((ipns, cid, node)),
        (_, Err(e)) => {
            ConsoleService::error(&format!("{:?}", e));
            None
        }
    }
}
