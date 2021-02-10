use crate::app::Route;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use yew_router::components::RouterAnchor;

type Anchor = RouterAnchor<Route>;

#[derive(Properties, Clone)]
pub struct Navbar {
    pub ens_name: String,
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
        html! {
            <div class="nav_background">
                <nav>
                    <Anchor route=Route::Live(self.ens_name.clone()) classes="navbar_tab">
                        <div>{"Live Stream"}</div>
                    </Anchor>
                    <Anchor route=Route::VideoList(self.ens_name.clone()) classes="navbar_tab">
                        <div>{"Videos"}</div>
                    </Anchor>
                </nav>
            </div>
        }
    }
}
