use crate::components::Navbar;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct Home {}

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
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
                    <h3 class="home_text"> { "v0.1.0 (Alpha)" } </h3>
                    <p class="home_text"> { "Defluencer.eth is a decentralized web application that allow anyone to live stream and host videos." } </p>
                    <p class="home_text"> { "Most browsers cannot access the decentralized web yet, for now you will need the inter-planetary file system and access to the Ethereum blockchain via Metamask." } </p>
                    <a class="home_text" href="https://docs.ipfs.io/install/ipfs-desktop/#ipfs-desktop"> { "IPFS Desktop" } </a>
                    <a class="home_text" href="https://docs.ipfs.io/install/ipfs-companion/#ipfs-companion"> { "IPFS Companion" } </a>
                    <a class="home_text" href="https://metamask.io/"> { "Metamask" } </a>
                    <a class="home_text" href="https://github.com/SionoiS/dit"> { "I'm building this app on Github join me there!" } </a>
                </div>
            </div>
        }
    }
}
