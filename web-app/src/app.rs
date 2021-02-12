use crate::pages::{Defluencer, Home, LiveStream, Settings, Video, VideoOnDemand};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::{Route, Router, Switch};

use cid::Cid;

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/#/video/{cid}"]
    Video(Cid),

    #[to = "/#/settings"]
    Settings,

    #[to = "/#/{ens_name}/videos"]
    VideoList(String),

    #[to = "/#/{ens_name}/live"]
    Live(String),

    #[to = "/#/{ens_name}"]
    Defluencer(String),

    #[to = "/"]
    Home,
}

impl AppRoute {
    pub fn into_route(self) -> Route {
        Route::from(self)
    }
}

pub struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Router<AppRoute>
                    render = Router::render(move |switch: AppRoute| {
                        match switch {
                            AppRoute::Video(cid) => html! { <Video metadata_cid=cid /> },
                            AppRoute::Settings => html! { <Settings /> },
                            AppRoute::VideoList(name) => html! { <VideoOnDemand ens_name=name /> },
                            AppRoute::Live(name) => html! { <LiveStream ens_name=name /> },
                            AppRoute::Defluencer(name) => html! { <Defluencer ens_name=name /> },
                            AppRoute::Home => html! { <Home /> },
                        }
                    })
                />
            </>
        }
    }
}
