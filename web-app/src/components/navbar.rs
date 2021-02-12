use crate::app::AppRoute;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use yew::ChangeData;

use yew_router::agent::{RouteAgentDispatcher, RouteRequest};
use yew_router::components::RouterAnchor;

type Anchor = RouterAnchor<AppRoute>;

pub struct Navbar {
    link: ComponentLink<Self>,

    ens_name: String,

    route_dispatcher: RouteAgentDispatcher,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ens_name: String,
}

pub enum Msg {
    Addrs(ChangeData),
}

impl Component for Navbar {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ens_name = props.ens_name;

        Self {
            link,
            ens_name,
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
                    {
                        if  self.ens_name.is_empty() {
                            html! {}
                        } else {
                            html! {
                                <>
                                <Anchor route=AppRoute::Live(self.ens_name.clone()) classes="navbar_tab">
                                    <div>{"Live Stream"}</div>
                                </Anchor>
                                <Anchor route=AppRoute::VideoList(self.ens_name.clone()) classes="navbar_tab">
                                    <div>{"Videos"}</div>
                                </Anchor>
                                </>
                            }
                        }
                    }
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
