use std::rc::Rc;
use std::str;

use crate::components::chat::display::Display;
use crate::components::chat::inputs::Inputs;
use crate::utils::web3::Web3Service;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct ChatWindow {
    topic: Rc<str>,
    web3: Web3Service,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub web3: Web3Service,
    pub topic: String,
}

impl Component for ChatWindow {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let web3 = props.web3;
        let topic = Rc::from(props.topic);

        Self { topic, web3 }
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
            <Display topic=self.topic.clone() />
            <Inputs topic=self.topic.clone() web3=self.web3.clone() />
        </div>
        }
    }
}
