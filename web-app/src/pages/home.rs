//use crate::app::Route;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

//use yew_router::components::RouterAnchor;

//type Anchor = RouterAnchor<Route>;

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
                /* <Anchor route=Route::Settings >
                    <div> { "Settings" } </div>
                </Anchor> */
                <div class="home_description">
                    <h1> { "Defluencers" } </h1>
                    <h2> { "Become Sovereign" } </h2>
                    <p> { "Defluencer.eth is native to the decentralized web. To use it you will need the inter-planetary file system (IPFS) and access to the Ethereum blockchain." } </p>
                    <p> { "IPFS is to the web 3.0 what HTTP is to the web 2.0. One of the difference between them is that HTTP ask for a location, where IPFS ask for content. It allow anyone and everyone on the web 3.0 to provide, in part or in full, the content you need. The decentralized web is more resilient, works offline and also on other planets!" } </p>
                    <p> { "Metamask will allow you to access the ethereum blockchain and it is used to associate human readable names with IPFS content identifiers." } </p>
                </div>
            </div>
        }
    }
}
