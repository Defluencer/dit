use crate::components::{Home, LiveStreamPlayer, Navbar, VODPlayer};
use crate::routing::Route;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::*;

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
                            Route::Live => html! {<LiveStreamPlayer />},
                            Route::Video => html! {<VODPlayer />},
                            Route::Home => html! {<Home />},
                        }
                    })
                />
            </>
        }
    }
}
