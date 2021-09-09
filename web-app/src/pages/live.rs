use std::rc::Rc;

use crate::components::{ChatWindow, Navbar, VideoPlayer};
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::Beacon;
use linked_data::moderation::{Bans, Moderators};

use either::Either;

/// Page displaying live video and chat.
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
        #[cfg(debug_assertions)]
        ConsoleService::info("Live Page Created");

        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&props.beacon, &self.beacon)
            || !Rc::ptr_eq(&props.bans, &self.bans)
            || !Rc::ptr_eq(&props.mods, &self.mods)
        {
            *self = props;

            #[cfg(debug_assertions)]
            ConsoleService::info("Live Page Changed");

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Navbar />
                <ybc::Section>
                    <ybc::Columns>
                        <ybc::Column>
                            <ybc::Box>
                                <VideoPlayer ipfs=self.ipfs.clone() beacon_or_metadata=Either::Left(self.beacon.clone()) />
                            </ybc::Box>
                        </ybc::Column>
                        <ybc::Column classes=classes!("is-one-fifth") >
                            <ChatWindow ipfs=self.ipfs.clone() web3=self.web3.clone() storage=self.storage.clone() beacon=self.beacon.clone() bans=self.bans.clone() mods=self.mods.clone() />
                        </ybc::Column>
                    </ybc::Columns>
                </ybc::Section>
            </>
        }
    }
}
