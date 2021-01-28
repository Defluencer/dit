use crate::components::Navbar;
use crate::pages::{Home, LiveStream, Video, VideoOnDemand};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::{Router, Switch};

use cid::Cid;

#[derive(Switch, Debug, Clone)]
pub enum Route {
    #[to = "/video/{cid}"]
    Video(Cid),

    #[to = "/videos"]
    Videos,

    #[to = "/live"]
    Live,

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
                <Navbar />
                <Router<Route>
                    render = Router::render(move |switch: Route| {
                        match switch {
                            Route::Live => html! {<LiveStream />},
                            Route::Videos => html! {<VideoOnDemand />},
                            Route::Home => html! {<Home />},
                            Route::Video(cid) => html! {<Video metadata_cid=cid />}
                        }
                    })
                />
            </>
        }
    }
}
