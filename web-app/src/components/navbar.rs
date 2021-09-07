use crate::app::AppRoute;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender, classes};
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
            <ybc::NavbarItem>
                <Anchor classes="has-text-white" route=AppRoute::Home>
                    {"Defluencer"}
                </Anchor>
            </ybc::NavbarItem>
        };

        let start = html! {
            <>
            <ybc::NavbarItem tab=true >
                <Anchor classes="has-text-white" route=AppRoute::Feed>
                    {"Content Feed"}
                </Anchor>
            </ybc::NavbarItem>
            <ybc::NavbarItem tab=true >
                <Anchor classes="has-text-white" route=AppRoute::Live>
                    {"Live"}
                </Anchor>
            </ybc::NavbarItem>
            </>
        };

        let end = html! {
            <ybc::NavbarItem tab=true >
                <Anchor classes="has-text-white" route=AppRoute::Settings>
                    {"Settings"}
                </Anchor>
            </ybc::NavbarItem>
        };

        html! {
            <ybc::Navbar classes=classes!("is-dark") transparent=false spaced=true padded=false navbrand=brand navstart=start navend=end navburger=true />
        }
    }
}
