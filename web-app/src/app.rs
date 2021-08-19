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
use linked_data::comments::Commentary;
use linked_data::feed::FeedAnchor;
use linked_data::friends::Friendlies;
use linked_data::moderation::Bans;
use linked_data::moderation::Moderators;

use either::Either;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

    name_cb: Callback<Result<Cid>>,

    beacon_set: HashSet<Cid>,
    beacon_cid: Option<Cid>,
    beacon: Option<Rc<Beacon>>,
    beacon_cb: Callback<Result<Beacon>>,

    feed_set: HashMap<String, Cid>,
    feed: Rc<FeedAnchor>,
    feed_cb: Callback<Result<(String, Cid, FeedAnchor)>>,

    bans_cid: Option<Cid>,
    bans: Rc<Bans>,
    bans_cb: Callback<Result<(String, Cid, Bans)>>,

    mods_cid: Option<Cid>,
    mods: Rc<Moderators>,
    mods_cb: Callback<Result<(String, Cid, Moderators)>>,

    comments_set: HashMap<String, Cid>,
    comments: Rc<Commentary>,
    comments_cb: Callback<Result<(String, Cid, Commentary)>>,

    friends_cid: Option<Cid>,
    friends: Rc<Friendlies>,
    friends_cb: Callback<Result<(String, Cid, Friendlies)>>,
}

pub enum AppMsg {
    ENSResolve(Result<Cid>),
    Beacon(Result<Beacon>),
    Feed(Result<(String, Cid, FeedAnchor)>),
    Bans(Result<(String, Cid, Bans)>),
    Mods(Result<(String, Cid, Moderators)>),
    Comments(Result<(String, Cid, Commentary)>),
    Friends(Result<(String, Cid, Friendlies)>),
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
        let name_cb = link.callback(AppMsg::ENSResolve);

        spawn_local({
            let cb = name_cb.clone();
            let web3 = props.web3.clone();
            let name = props.ens_name.to_string();

            async move { cb.emit(web3.get_ipfs_content(name).await) }
        });

        let beacon_cb = link.callback(AppMsg::Beacon);

        if let Some(cid) = props.storage.get_cid(&props.ens_name) {
            spawn_local({
                let cb = beacon_cb.clone();
                let ipfs = props.ipfs.clone();

                async move { cb.emit(ipfs.dag_get(cid, Option::<String>::None).await) }
            });
        }

        Self {
            props,

            name_cb,

            beacon_set: HashSet::with_capacity(10),
            beacon_cid: None,
            beacon: None,
            beacon_cb,

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
            comments: Rc::from(Commentary::default()),
            comments_cb: link.callback(AppMsg::Comments),

            friends_cid: None,
            friends: Rc::from(Friendlies::default()),
            friends_cb: link.callback(AppMsg::Friends),
        }
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
                            AppRoute::Content(cid) => html! { <Content ipfs=ipfs.clone() metadata_cid=cid comments=comments.clone() /> },
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
    /// Callback when Ethereum Name Service resolve any name.
    fn on_name(&mut self, response: Result<Cid>) -> bool {
        let beacon_cid = match response {
            Ok(cid) => cid,
            Err(e) => {
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

            async move { cb.emit(ipfs.dag_get(beacon_cid, Option::<String>::None).await) }
        });

        if self.beacon_cid.is_none() {
            self.beacon_cid = beacon_cid.into();

            self.props
                .storage
                .set_beacon(&self.props.ens_name, &beacon_cid);
        }

        self.beacon_set.insert(beacon_cid);

        false
    }

    /// Callback when IPFS dag get return any beacon.
    fn on_beacon(&mut self, response: Result<Beacon>) -> bool {
        let beacon = match response {
            Ok(beacon) => beacon,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        spawn_local({
            let cb = self.feed_cb.clone();
            let ipfs = self.props.ipfs.clone();
            let feed = beacon.content_feed.clone();

            async move { cb.emit(ipfs.resolve_and_dag_get(feed).await) }
        });

        if let Some(cid) = self.props.storage.get_cid(&beacon.content_feed) {
            spawn_local({
                let cb = self.feed_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let feed = beacon.content_feed.clone();

                async move {
                    match ipfs.dag_get(cid, Option::<&str>::None).await {
                        Ok(node) => cb.emit(Ok((feed, cid, node))),
                        Err(e) => cb.emit(Err(e)),
                    }
                }
            });
        }

        if let Some(comments) = beacon.comments.as_ref() {
            spawn_local({
                let cb = self.comments_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let comments = comments.clone();

                async move { cb.emit(ipfs.resolve_and_dag_get(comments).await) }
            });

            if let Some(cid) = self.props.storage.get_cid(comments) {
                spawn_local({
                    let cb = self.comments_cb.clone();
                    let ipfs = self.props.ipfs.clone();
                    let comments = comments.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit(Ok((comments, cid, node))),
                            Err(e) => cb.emit(Err(e)),
                        }
                    }
                });
            }
        }

        if self.beacon.is_some() {
            //Only resolve the feed and comments of your friends.
            return false;
        }

        if let Some(friends) = beacon.friends.as_ref() {
            spawn_local({
                let cb = self.friends_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let friends = friends.clone();

                async move { cb.emit(ipfs.resolve_and_dag_get(friends).await) }
            });

            if let Some(cid) = self.props.storage.get_cid(friends) {
                spawn_local({
                    let cb = self.friends_cb.clone();
                    let ipfs = self.props.ipfs.clone();
                    let friends = friends.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit(Ok((friends, cid, node))),
                            Err(e) => cb.emit(Err(e)),
                        }
                    }
                });
            }
        }

        if let Some(bans) = beacon.bans.as_ref() {
            spawn_local({
                let cb = self.bans_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let bans = bans.clone();

                async move { cb.emit(ipfs.resolve_and_dag_get(bans).await) }
            });

            if let Some(cid) = self.props.storage.get_cid(bans) {
                spawn_local({
                    let cb = self.bans_cb.clone();
                    let ipfs = self.props.ipfs.clone();
                    let bans = bans.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit(Ok((bans, cid, node))),
                            Err(e) => cb.emit(Err(e)),
                        }
                    }
                });
            }
        }

        if let Some(mods) = beacon.mods.as_ref() {
            spawn_local({
                let cb = self.mods_cb.clone();
                let ipfs = self.props.ipfs.clone();
                let mods = mods.clone();

                async move { cb.emit(ipfs.resolve_and_dag_get(mods).await) }
            });

            if let Some(cid) = self.props.storage.get_cid(mods) {
                spawn_local({
                    let cb = self.mods_cb.clone();
                    let ipfs = self.props.ipfs.clone();
                    let mods = mods.clone();

                    async move {
                        match ipfs.dag_get(cid, Option::<&str>::None).await {
                            Ok(node) => cb.emit(Ok((mods, cid, node))),
                            Err(e) => cb.emit(Err(e)),
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
    fn on_feed(&mut self, res: Result<(String, Cid, FeedAnchor)>) -> bool {
        let (ipns, feed_cid, mut feed) = match res {
            Ok((ipns, cid, feed)) => (ipns, cid, feed),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if Some(&feed_cid) == self.feed_set.get(&ipns) {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Content Feed Update");

        Rc::make_mut(&mut self.feed)
            .content
            .append(&mut feed.content);

        self.props.storage.set_cid(&ipns, &feed_cid);

        self.feed_set.insert(ipns, feed_cid);

        true
    }

    /// Callback when IPFS dag get return any comments.
    fn on_comments(&mut self, result: Result<(String, Cid, Commentary)>) -> bool {
        let (ipns, comments_cid, comments) = match result {
            Ok((ipns, cid, node)) => (ipns, cid, node),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if Some(&comments_cid) == self.comments_set.get(&ipns) {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Comment List Update");

        Rc::make_mut(&mut self.comments).merge(comments);

        self.props.storage.set_cid(&ipns, &comments_cid);

        self.comments_set.insert(ipns, comments_cid);

        true
    }

    /// Callback when IPFS dag get return your ban list.
    fn on_ban_list(&mut self, result: Result<(String, Cid, Bans)>) -> bool {
        let (ipns, bans_cid, bans) = match result {
            Ok((ipns, bans_cid, bans)) => (ipns, bans_cid, bans),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if Some(bans_cid) == self.bans_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Ban List Update");

        self.props.storage.set_cid(&ipns, &bans_cid);

        self.bans_cid = bans_cid.into();
        self.bans = Rc::from(bans);

        true
    }

    /// Callback when IPFS dag get return your moderators.
    fn on_mod_list(&mut self, result: Result<(String, Cid, Moderators)>) -> bool {
        let (ipns, mods_cid, mods) = match result {
            Ok((ipns, mods_cid, mods)) => (ipns, mods_cid, mods),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        if Some(mods_cid) == self.mods_cid {
            return false;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info("Moderator List Update");

        self.props.storage.set_cid(&ipns, &mods_cid);

        self.mods_cid = mods_cid.into();
        self.mods = Rc::from(mods);

        true
    }

    /// Callback when IPFS dag get return your friend list.
    fn on_friends(&mut self, result: Result<(String, Cid, Friendlies)>) -> bool {
        let (ipns, friends_cid, friends) = match result {
            Ok((ipns, cid, node)) => (ipns, cid, node),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
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
                        let ipfs = self.props.ipfs.clone();
                        let cid = ipld.link;
                        let beacon_cb = self.beacon_cb.clone();

                        async move { beacon_cb.emit(ipfs.dag_get(cid, Option::<&str>::None).await) }
                    });
                }
                Either::Left(name) => {
                    spawn_local({
                        let web3 = self.props.web3.clone();
                        let name = name.clone();
                        let name_cb = self.name_cb.clone();

                        async move { name_cb.emit(web3.get_ipfs_content(name).await) }
                    });
                }
            }
        }

        self.props.storage.set_cid(&ipns, &friends_cid);

        self.friends_cid = friends_cid.into();
        self.friends = Rc::from(friends);

        false
    }
}
