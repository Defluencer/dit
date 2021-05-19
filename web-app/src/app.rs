use crate::pages::{Home, Live, Settings, Video, Videos};
use crate::utils::ipfs::IpfsService;
use crate::utils::web3::Web3Service;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew_router::prelude::{Router, Switch};

use cid::Cid;

pub const ENS_NAME: &str = "sionois";

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/#/video/{cid}"]
    Video(Cid),

    #[to = "/#/settings"]
    Settings,

    #[to = "/#/live"]
    Live,

    #[to = "/#/videos"]
    Videos,

    #[to = "/"]
    Home,
}

pub struct App {
    web3: Web3Service,
    ipfs: IpfsService,
}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _: ComponentLink<Self>) -> Self {
        let web3 = Web3Service::new().unwrap();
        let ipfs = IpfsService::new();

        Self { web3, ipfs }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let web3 = self.web3.clone();
        let ipfs = self.ipfs.clone();

        html! {
            <>
                <Router<AppRoute>
                    render = Router::render(move |switch: AppRoute| {
                        match switch {
                            AppRoute::Video(cid) => html! { <Video ipfs=ipfs.clone() metadata_cid=cid /> },
                            AppRoute::Settings => html! { <Settings /> },
                            AppRoute::Live => html! { <Live ipfs=ipfs.clone() web3=web3.clone() /> },
                            AppRoute::Videos => html! { <Videos ipfs=ipfs.clone() web3=web3.clone() /> },
                            AppRoute::Home => html! { <Home /> },
                        }
                    })
                />
            </>
        }
    }
}
