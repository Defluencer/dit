use crate::app::AppRoute;
use crate::components::Navbar;

use yew::prelude::{classes, html, Component, ComponentLink, Html, ShouldRender};

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
        let body = html! {
            <ybc::Container classes=classes!("has-text-centered") >
                <ybc::Title>
                {
                    "Build social media applications and websites on the web 3.0"
                }
                </ybc::Title>
                <ybc::Subtitle>
                {
                    "Defluencer is a protocol for decentralized social media.
                    With immutable and interoperable content, your data cannot be changed and will follow you to any app or website built on the protocol.
                    Your an influencer? Build your own website to show the world you awesome content!
                    Do you love wombats? Build a platform and agregate the world's content on the subject!"
                }
                </ybc::Subtitle>
                <ybc::ButtonRouter<AppRoute> route=AppRoute::Home >
                    {"Get Started"}
                </ybc::ButtonRouter<AppRoute>>
            </ybc::Container>
        };

        html! {
            <>
                <Navbar />
                <ybc::Hero body=body />
                <ybc::Content>
                    <p> { "Defluencer.eth is a demo website that showcase decentralized social media (live streaming, live chat, blog and video hosting)." } </p>
                    <p> { "Most web browsers cannot access the decentralized web yet, for now you will need Metamask and IPFS with C.O.R.S. & PubSub enabled." } </p>
                </ybc::Content>
                <ybc::Footer>
                    <ybc::Tabs boxed=true fullwidth=true>
                        <ul>
                            <li><a href="https://docs.ipfs.io/install/ipfs-desktop/#ipfs-desktop"> { "IPFS Desktop" } </a></li>
                            <li><a href="https://metamask.io/"> { "Metamask" } </a></li>
                            <li><a href="https://github.com/SionoiS/dit"> { "Github" } </a></li>
                        </ul>
                    </ybc::Tabs>
                </ybc::Footer>
            </>
        }
    }
}
