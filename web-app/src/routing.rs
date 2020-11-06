use yew_router::prelude::*;

#[derive(Switch, Debug, Clone)]
pub enum Route {
    #[to = "/vod"]
    Video,

    #[to = "/live"]
    Live,

    #[to = "/"]
    Home,
}
