use std::rc::Rc;
use std::str;

use crate::components::chat::display::Display;
use crate::components::chat::inputs::Inputs;
use crate::utils::{IpfsService, LocalStorage, Web3Service};

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct ChatWindow {
    topic: Rc<str>,
    ban_list: Rc<str>,
    mod_list: Rc<str>,
    web3: Web3Service,
    ipfs: IpfsService,
    storage: LocalStorage,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub web3: Web3Service,
    pub ipfs: IpfsService,
    pub storage: LocalStorage,
    pub topic: Rc<str>,
    pub ban_list: Rc<str>,
    pub mod_list: Rc<str>,
}

impl Component for ChatWindow {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let Props {
            ipfs,
            web3,
            storage,
            topic,
            ban_list,
            mod_list,
        } = props;

        Self {
            topic,
            ban_list,
            mod_list,
            web3,
            ipfs,
            storage,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
        <div class="chat_window">
            <Display ipfs=self.ipfs.clone() topic=self.topic.clone() ban_list=self.ban_list.clone() mod_list=self.mod_list.clone() />
            <Inputs ipfs=self.ipfs.clone() web3=self.web3.clone() storage=self.storage.clone()  topic=self.topic.clone() />
        </div>
        }
    }
}
