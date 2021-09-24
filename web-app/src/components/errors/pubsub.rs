//use crate::app::AppRoute;

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

/// Error indicator.
#[derive(Clone, Properties)]
pub struct IPFSPubSubError {}

impl Component for IPFSPubSubError {
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
                { "Please verify that PubSub is enabled." }
                </ybc::Subtitle>
            </ybc::Container>
        }
    }
}
