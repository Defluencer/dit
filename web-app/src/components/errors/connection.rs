use crate::app::AppRoute;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

/// Error indicator.
#[derive(Clone, Properties)]
pub struct IPFSConnectionError {}

impl Component for IPFSConnectionError {
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
        html! {
            <ybc::Container classes=classes!("has-text-centered") >
                <ybc::Title size=ybc::HeaderSize::Is5 >
                    { "An Error was encounted." }
                </ybc::Title>
                <ybc::Subtitle size=ybc::HeaderSize::Is6 >
                { "Please verify your connection to IPFS" }
                </ybc::Subtitle>
                <ybc::ButtonRouter<AppRoute> route=AppRoute::Settings classes=classes!("is-primary") >
                    {"Go to settings"}
                </ybc::ButtonRouter<AppRoute>>
            </ybc::Container>
        }
    }
}
