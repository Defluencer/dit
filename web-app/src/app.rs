use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::pages::{Content, ContentFeed, Home, Live, Settings};
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;
use yew_router::prelude::{Router, Switch};

use linked_data::beacon::Beacon;
use linked_data::comments::{CommentCache, Commentary};
use linked_data::feed::FeedAnchor;
use linked_data::friends::Friendlies;
use linked_data::moderation::Bans;
use linked_data::moderation::Moderators;

use either::Either;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type CommentCallback = Callback<(String, Cid, Result<(Cid, Commentary)>)>;

#[derive(Switch, Debug, Clone)]
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

    name_cb: Callback<(String, Result<Cid>)>,

    beacon_set: HashSet<Cid>,
    beacon: Option<Rc<Beacon>>,
    beacon_cb: Callback<(Cid, Result<Beacon>)>,

    /// Maps IPNS to FeedAnchors
    feed_set: HashMap<Cid, Cid>,
    feed: Rc<FeedAnchor>,
    feed_cb: Callback<(Cid, Result<(Cid, FeedAnchor)>)>,

    bans_cid: Option<Cid>,
    bans: Rc<Bans>,
    bans_cb: Callback<(Cid, Result<(Cid, Bans)>)>,

    mods_cid: Option<Cid>,
    mods: Rc<Moderators>,
    mods_cb: Callback<(Cid, Result<(Cid, Moderators)>)>,

    /// Maps IPNS to Commentary
    comments_set: HashMap<Cid, Cid>,
    comments: Rc<CommentCache>,
    comments_cb: CommentCallback,

    friends_cid: Option<Cid>,
    friends: Rc<Friendlies>,
    friends_cb: Callback<(Cid, Result<(Cid, Friendlies)>)>,
}

#[allow(clippy::large_enum_variant)]
pub enum AppMsg {
    ENSResolve((String, Result<Cid>)),
    Beacon((Cid, Result<Beacon>)),
    Feed((Cid, Result<(Cid, FeedAnchor)>)),
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
    pub ens_name: Rc<str>,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let app = Self {
            props,

            name_cb: link.callback(AppMsg::ENSResolve),

            beacon_set: HashSet::with_capacity(10),
            beacon: None,
            beacon_cb: link.callback(AppMsg::Beacon),

            feed_set: HashMap::with_capacity(10),
            feed: Rc::from(FeedAnchor::default()),
            feed_cb: link.callback(AppMsg::Feed),

            bans_cid: None,
            bans: Rc::from(Bans::default()),
            bans_cb: link.callback(AppMsg::Bans),

            mods_cid: None,
            mods: Rc::from(Moderators::default()),
            mods_cb: link.callback(AppMsg::Mods),

            comments_set: HashMap::with_capacity(10),
            comments: Rc::from(CommentCache::create()),
            comments_cb: link.callback(AppMsg::Comments),

            friends_cid: None,
            friends: Rc::from(Friendlies::default()),
            friends_cb: link.callback(AppMsg::Friends),
        };

        app.get_beacon(&app.props.ens_name);

        app
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
        let web3 = self.props.web3.clone();
        let ipfs = self.props.ipfs.clone();
        let storage = self.props.storage.clone();
        let feed = self.feed.clone();
        let beacon = self.beacon.clone().unwrap_or_default();
        let bans = self.bans.clone();
        let mods = self.mods.clone();
        let comments = self.comments.clone();
        //let friends = self.friends.clone();

        html! {
            <>
                <Router<AppRoute>
                    render = Router::render(move |switch: AppRoute| {
                        match switch {
                            AppRoute::Content(cid) => html! { <Content ipfs=ipfs.clone() cid=cid comments=comments.clone() /> },
                            AppRoute::Settings => html! { <Settings storage=storage.clone() /> },
                            AppRoute::Live => html! { <Live ipfs=ipfs.clone() web3=web3.clone() storage=storage.clone() beacon=beacon.clone() bans=bans.clone() mods=mods.clone() /> },
                            AppRoute::Feed => html! { <ContentFeed ipfs=ipfs.clone() storage=storage.clone() feed=feed.clone() comments=comments.clone() /> },
                            AppRoute::Home => html! { <Home /> },
                        }
                    })
                />
            </>
        }
    }
}

impl App {
    /// Resolve ENS name and check local storage.
    fn get_beacon(&self, ens_name: &str) {
        spawn_local({
            let cb = self.name_cb.clone();
            let web3 = self.props.web3.clone();
            let name = ens_name.to_owned();

            async move { cb.emit((name.clone(), web3.get_ipfs_content(name).await)) }
        });

        if let Some(cid) = self.props.storage.get_cid(ens_name) {
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

        #[cfg(debug_assertions)]
        ConsoleService::info("Name Resolve");

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

        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        spawn_local({
            let cb = self.feed_cb.clone();
            let ipfs = self.props.ipfs.clone();
            let feed = beacon.content_feed;

            async move { cb.emit((feed, ipfs.resolve_and_dag_get(feed).await)) }
        });

        if let Some(cid) = self.props.storage.get_cid(&beacon.content_feed.to_string()) {
            spawn_local({
                let cb = self.feed_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let feed = beacon.content_feed;

                async move {
                    match ipfs.dag_get(cid, Option::<&str>::None).await {
                        Ok(node) => cb.emit((feed, Ok((cid, node)))),
                        Err(e) => cb.emit((feed, Err(e))),
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

        false
    }

    /// Callback when IPFS dag get return any content feed.
    fn on_feed(&mut self, res: (Cid, Result<(Cid, FeedAnchor)>)) -> bool {
        let (ipns, feed_cid, mut feed) = match on_node(res) {
            Some(res) => res,
            None => return false,
        };

        if Some(feed_cid) == self.feed_set.insert(ipns, feed_cid) {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Feed Update");

        Rc::make_mut(&mut self.feed)
            .content
            .append(&mut feed.content);

        self.props.storage.set_cid(&ipns.to_string(), &feed_cid);

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

        #[cfg(debug_assertions)]
        ConsoleService::info("Comment List Update");

        Rc::make_mut(&mut self.comments).insert(name, comments);

        self.props.storage.set_cid(&ipns.to_string(), &comments_cid);

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

        #[cfg(debug_assertions)]
        ConsoleService::info("Ban List Update");

        self.props.storage.set_cid(&ipns.to_string(), &bans_cid);

        self.bans_cid = bans_cid.into();
        self.bans = Rc::from(bans);

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

        #[cfg(debug_assertions)]
        ConsoleService::info("Moderator List Update");

        self.props.storage.set_cid(&ipns.to_string(), &mods_cid);

        self.mods_cid = mods_cid.into();
        self.mods = Rc::from(mods);

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

        #[cfg(debug_assertions)]
        ConsoleService::info("Friend List Update");

        for friend in friends.list.iter() {
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
