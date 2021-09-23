use crate::app::AppRoute;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

type Anchor = RouterAnchor<AppRoute>;

/// Navigation bar.
#[derive(Properties, Clone)]
pub struct Navbar {}

impl Component for Navbar {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let brand = html! {
            <Anchor classes="navbar-item" route=AppRoute::Home>
                //<img src="/img/defluencer_logo.svg" alt="defluencer-logo" width="133" height="56" />
                //{"Defluencer"}
                <span class="icon-text">
                    <span class="icon"><i class="fas fa-home"></i></span>
                    <span> {"Home"} </span>
                </span>
            </Anchor>
        };

        let start = html! {
            <>
                <Anchor classes="navbar-item" route=AppRoute::Feed>
                    <span class="icon-text">
                        <span class="icon"><i class="fas fa-rss"></i></span>
                        <span> {"Content Feed"} </span>
                    </span>
                </Anchor>
                <Anchor classes="navbar-item" route=AppRoute::Live>
                    <span class="icon-text">
                        <span class="icon"><i class="fas fa-broadcast-tower"></i></span>
                        <span> {"Live"} </span>
                    </span>
                </Anchor>
            </>
        };

        let end = html! {
            <Anchor classes="navbar-item" route=AppRoute::Settings>
                <span class="icon-text" >
                    <span class="icon"><i class="fas fa-cog"></i></span>
                    <span> {"Settings"} </span>
                </span>
            </Anchor>
        };

        html! {
            <ybc::Navbar classes=classes!("is-spaced") transparent=false spaced=true padded=false navbrand=brand navstart=start navend=end navburger=true />
        }
    }
}
