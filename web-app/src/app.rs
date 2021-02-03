use crate::pages::{Defluencer, Home, LiveStream, Video, VideoOnDemand};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::{Router, Switch};

use cid::Cid;

#[derive(Switch, Debug, Clone)]
pub enum Route {
    #[to = "/video/{cid}"]
    Video(Cid),

    #[to = "/{cid}/videos"]
    VideoList(Cid), // cid for now and ENS later

    #[to = "/{cid}/live"]
    Live(Cid), // cid for now and ENS later

    #[to = "/{cid}"]
    Defluencer(Cid), // cid for now and ENS later

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
                            Route::Live(cid) => html! {<LiveStream beacon_cid=cid />},
                            Route::VideoList(cid) => html! {<VideoOnDemand beacon_cid=cid />},
                            Route::Defluencer(cid) => html! { <Defluencer beacon_cid=cid /> },
                            Route::Home => html! {<Home />},
                            Route::Video(cid) => html! {<Video metadata_cid=cid />}
                        }
                    })
                />
            </>
        }
    }
}
