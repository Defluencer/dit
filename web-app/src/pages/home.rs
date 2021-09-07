use crate::app::AppRoute;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender, classes};
use yew_router::components::RouterAnchor;

type Anchor = RouterAnchor<AppRoute>;

/// The landing page.
pub struct Home {}

impl Component for Home {
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
        let head = html! {
            <ybc::Level>
                <ybc::LevelItem classes=classes!("hash-text-centered") >
                    <Anchor classes="has-text-white" route=AppRoute::Home>
                        {"Home"}
                    </Anchor>
                </ybc::LevelItem>
                <ybc::LevelItem classes=classes!("hash-text-centered") >
                    <Anchor classes="has-text-white" route=AppRoute::Feed>
                        {"Feed"}
                    </Anchor>
                </ybc::LevelItem>
                <ybc::LevelItem classes=classes!("hash-text-centered") >
                    <ybc::Image classes=classes!("is-32x32") size=ybc::ImageSize::Is1by1 >
                        <img class="is-rounded" src="../images/logo.png" alt="Defluencer Logo" />
                    </ybc::Image>
                </ybc::LevelItem>
                <ybc::LevelItem classes=classes!("hash-text-centered") >
                    <Anchor classes="has-text-white" route=AppRoute::Live>
                        {"Live"}
                    </Anchor>
                </ybc::LevelItem>
                <ybc::LevelItem classes=classes!("hash-text-centered") >
                    <Anchor classes="has-text-white" route=AppRoute::Settings>
                        {"Settings"}
                    </Anchor>
                </ybc::LevelItem>
            </ybc::Level>
        };

        let body = html!{
            <ybc::Container classes=classes!("has-text-centered") >
                <ybc::Title> { "Defluencers" } </ybc::Title>
                <ybc::Subtitle> { "Become Sovereign!" } </ybc::Subtitle>
                <p class=classes!("block")> { "Defluencer.eth is a demo website that showcase decentralized social media (live streaming, live chat, blog and video hosting)." } </p>
                <p class=classes!("block")> { "Most web browsers cannot access the decentralized web yet, for now you will need Metamask and IPFS with C.O.R.S. & PubSub enabled." } </p>
            </ybc::Container>
        };

        let foot = html! {
            <ybc::Tabs boxed=true fullwidth=true>
                <ul>
                    <li><a href="https://docs.ipfs.io/install/ipfs-desktop/#ipfs-desktop"> { "IPFS Desktop" } </a></li>
                    <li><a href="https://metamask.io/"> { "Metamask" } </a></li>
                    <li><a href="https://github.com/SionoiS/dit"> { "Github" } </a></li>
                </ul>
            </ybc::Tabs>
        };

        html! {
            <ybc::Hero classes=classes!("is-fullheight", "has-background-dark") head=head body=body foot=foot />
        }
    }
}
