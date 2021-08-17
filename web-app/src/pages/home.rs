use crate::components::Navbar;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

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
        html! {
            <div class="home_page">
                <Navbar />
                <div class="home_description">
                    <h1 class="home_text"> { "Defluencers" } </h1>
                    <h2 class="home_text"> { "Become Sovereign!" } </h2>
                    <p class="home_text"> { "Defluencer.eth is a demo website that showcase decentralized social media (live streaming, live chat, blog and video hosting)." } </p>
                    <p class="home_text"> { "Most web browsers cannot access the decentralized web yet, for now you will need Metamask and IPFS with C.O.R.S. & PubSub enabled." } </p>
                    <a class="home_text" href="https://docs.ipfs.io/install/ipfs-desktop/#ipfs-desktop"> { "IPFS Desktop" } </a>
                    <a class="home_text" href="https://metamask.io/"> { "Metamask" } </a>
                    <a class="home_text" href="https://github.com/SionoiS/dit"> { "Source Code" } </a>
                </div>
            </div>
        }
    }
}
