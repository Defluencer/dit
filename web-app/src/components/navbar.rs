use crate::app::AppRoute;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

use yew::ChangeData;

use yew_router::agent::{RouteAgentDispatcher, RouteRequest};
use yew_router::components::RouterAnchor;

type Anchor = RouterAnchor<AppRoute>;

pub struct Navbar {
    link: ComponentLink<Self>,

    route_dispatcher: RouteAgentDispatcher,
}

pub enum Msg {
    Addrs(ChangeData),
}

impl Component for Navbar {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            route_dispatcher: RouteAgentDispatcher::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Addrs(msg) => self.addrs(msg),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="nav_background">
                <nav>
                    <Anchor route=AppRoute::Home classes="navbar_tab">
                        <div>{"Home"}</div>
                    </Anchor>
                    <div class="navbar_tab">
                        <label for="search"> { "Search" } </label>
                        <input type="text" id="search" name="search"
                        onchange=self.link.callback(Msg::Addrs)
                        />
                    </div>
                    <Anchor route=AppRoute::Settings classes="navbar_tab">
                        <div>{"Settings"}</div>
                    </Anchor>
                </nav>
            </div>
        }
    }
}

impl Navbar {
    fn addrs(&mut self, msg: ChangeData) -> bool {
        match msg {
            ChangeData::Value(search_value) => {
                let route = crate::app::AppRoute::Defluencer(search_value);

                self.route_dispatcher
                    .send(RouteRequest::ChangeRoute(route.into_route()));
            }
            ChangeData::Select(_) => {}
            ChangeData::Files(_) => {}
        }

        false
    }
}
