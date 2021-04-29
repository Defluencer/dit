use std::rc::Rc;
use std::str;

use crate::components::chat::display::Display;
use crate::components::chat::inputs::Inputs;
use crate::utils::ipfs::IpfsService;
use crate::utils::web3::Web3Service;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct ChatWindow {
    topic: Rc<str>,
    web3: Web3Service,
    ipfs: IpfsService,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub web3: Web3Service,
    pub ipfs: IpfsService,
    pub topic: String,
}

impl Component for ChatWindow {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let Props { ipfs, web3, topic } = props;

        let topic = Rc::from(topic);

        Self { topic, web3, ipfs }
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
            <Display ipfs=self.ipfs.clone() topic=self.topic.clone() />
            <Inputs ipfs=self.ipfs.clone() topic=self.topic.clone() web3=self.web3.clone() />
        </div>
        }
    }
}
