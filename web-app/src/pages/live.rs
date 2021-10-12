use std::rc::Rc;

use crate::components::{ChatWindow, IPFSConnectionError, Navbar, VideoPlayer};
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};

#[cfg(debug_assertions)]
use yew::services::ConsoleService;

use linked_data::live::Live;
use linked_data::moderation::{Bans, Moderators};

use either::Either;

/// Page displaying live video and chat.
#[derive(Properties, Clone)]
pub struct LivePage {
    pub peer_id: Rc<Option<String>>,
    pub ipfs: IpfsService,
    pub web3: Web3Service,
    pub storage: LocalStorage,
    pub live: Rc<Live>,
    pub mods: Rc<Moderators>,
    pub bans: Rc<Bans>,
}

impl Component for LivePage {
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
        if !Rc::ptr_eq(&props.live, &self.live)
            || !Rc::ptr_eq(&props.bans, &self.bans)
            || !Rc::ptr_eq(&props.mods, &self.mods)
            || !Rc::ptr_eq(&props.peer_id, &self.peer_id)
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
                {
                    if self.peer_id.is_none() {
                        html! { <IPFSConnectionError /> }
                    } else {
                        html! {
                        <ybc::Columns>
                            <ybc::Column>
                                <ybc::Box>
                                    <VideoPlayer ipfs=self.ipfs.clone() beacon_or_metadata=Either::Left(self.live.clone()) />
                                </ybc::Box>
                            </ybc::Column>
                            <ybc::Column classes=classes!("is-one-fifth") >
                                <ChatWindow ipfs=self.ipfs.clone() web3=self.web3.clone() storage=self.storage.clone() live=self.live.clone() bans=self.bans.clone() mods=self.mods.clone() />
                            </ybc::Column>
                        </ybc::Columns>
                        }
                    }
                }
                </ybc::Section>
            </>
        }
    }
}
