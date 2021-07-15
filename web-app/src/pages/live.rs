use std::rc::Rc;

use crate::components::{ChatWindow, Navbar, VideoPlayer};
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::beacon::Beacon;
use linked_data::moderation::{Bans, Moderators};

#[derive(Properties, Clone)]
pub struct Live {
    pub ipfs: IpfsService,
    pub web3: Web3Service,
    pub storage: LocalStorage,
    pub beacon: Rc<Beacon>,
    pub mods: Rc<Moderators>,
    pub bans: Rc<Bans>,
}

impl Component for Live {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.beacon != self.beacon || props.bans != self.bans || props.mods != self.mods {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="live_page">
                <Navbar />
                <div class="live_stream">
                    <VideoPlayer ipfs=self.ipfs.clone() beacon=self.beacon.clone() />
                    <ChatWindow ipfs=self.ipfs.clone() web3=self.web3.clone() storage=self.storage.clone() beacon=self.beacon.clone() bans=self.bans.clone() mods=self.mods.clone() />
                </div>
            </div>
        }
    }
}
