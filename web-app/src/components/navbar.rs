use crate::app::Route;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use yew_router::components::RouterAnchor;

use cid::Cid;

#[derive(Properties, Clone)]
pub struct Navbar {
    pub beacon_cid: Cid,
}

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
        type Anchor = RouterAnchor<Route>;

        html! {
            <div class="nav_background">
                <nav>
                    /* <Anchor route=Route::Home classes="navbar_tab">
                        <div>{"Home"}</div>
                    </Anchor> */
                    <Anchor route=Route::Live(self.beacon_cid) classes="navbar_tab">
                        <div>{"Live Stream"}</div>
                    </Anchor>
                    <Anchor route=Route::VideoList(self.beacon_cid) classes="navbar_tab">
                        <div>{"Videos"}</div>
                    </Anchor>
                </nav>
            </div>
        }
    }
}
