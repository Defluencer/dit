use crate::pages::{Defluencer, Home, LiveStream, Video, VideoOnDemand};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::{Router, Switch};

use cid::Cid;

#[derive(Switch, Debug, Clone)]
pub enum Route {
    #[to = "/video/{cid}"]
    Video(Cid),

    #[to = "/{ens_name}/videos"]
    VideoList(String),

    #[to = "/{ens_name}/live"]
    Live(String),

    #[to = "/{ens_name}"]
    Defluencer(String),

    #[to = "/"]
    Home,
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
                <Router<Route>
                    render = Router::render(move |switch: Route| {
                        match switch {
                            Route::Video(cid) => html! { <Video metadata_cid=cid /> },
                            Route::VideoList(name) => html! { <VideoOnDemand ens_name=name /> },
                            Route::Live(name) => html! { <LiveStream ens_name=name /> },
                            Route::Defluencer(name) => html! { <Defluencer ens_name=name /> },
                            Route::Home => html! { <Home /> },
                        }
                    })
                />
            </>
        }
    }
}
