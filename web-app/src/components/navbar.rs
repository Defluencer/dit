use crate::app::AppRoute;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

use yew_router::components::RouterAnchor;

type Anchor = RouterAnchor<AppRoute>;

pub struct Navbar {}

impl Component for Navbar {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
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
            <div class="nav_background">
                <nav>
                    <Anchor route=AppRoute::Home classes="navbar_tab">
                        <div>{"Home"}</div>
                    </Anchor>
                    <Anchor route=AppRoute::Feed classes="navbar_tab">
                        <div>{"Content Feed"}</div>
                    </Anchor>
                    <Anchor route=AppRoute::Live classes="navbar_tab">
                        <div>{"Live"}</div>
                    </Anchor>
                    <Anchor route=AppRoute::Settings classes="navbar_tab">
                        <div>{"Settings"}</div>
                    </Anchor>
                </nav>
            </div>
        }
    }
}
