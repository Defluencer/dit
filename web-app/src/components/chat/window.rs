use std::rc::Rc;

use crate::components::chat::display::Display;
use crate::components::chat::inputs::Inputs;
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::beacon::Beacon;
use linked_data::moderation::{Bans, Moderators};

/// Live chat widget
#[derive(Properties, Clone)]
pub struct ChatWindow {
    pub web3: Web3Service,
    pub ipfs: IpfsService,
    pub storage: LocalStorage,
    pub beacon: Rc<Beacon>,
    pub mods: Rc<Moderators>,
    pub bans: Rc<Bans>,
}

impl Component for ChatWindow {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&self.beacon, &props.beacon)
            || !Rc::ptr_eq(&self.mods, &props.mods)
            || !Rc::ptr_eq(&self.bans, &props.bans)
        {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
        <>
            <Display ipfs=self.ipfs.clone() beacon=self.beacon.clone() bans=self.bans.clone() mods=self.mods.clone() />
            <Inputs ipfs=self.ipfs.clone() web3=self.web3.clone() storage=self.storage.clone() beacon=self.beacon.clone() />
        </>
        }
    }
}
