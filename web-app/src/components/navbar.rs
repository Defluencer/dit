use crate::routing::Route;

use yew::prelude::*;

use yew_router::components::RouterAnchor;

pub struct Navbar {}

impl Component for Navbar {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
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
                    <Anchor route=Route::Home classes="navbar_tab">
                        <div>{"Live-Like"}</div>
                    </Anchor>
                    <Anchor route=Route::Live classes="navbar_tab">
                        <div>{"Live Stream"}</div>
                    </Anchor>
                    <Anchor route=Route::Video classes="navbar_tab">
                        <div>{"Video"}</div>
                    </Anchor>
                </nav>
            </div>
        }
    }
}
