use crate::components::chat::display::Display;
use crate::components::chat::inputs::Inputs;
use crate::utils::local_storage::get_local_storage;
use crate::utils::web3::Web3Service;

use web_sys::{Storage, Window};

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct ChatWindow {
    link: ComponentLink<Self>,
    topic: String,
    window: Window,
    storage: Option<Storage>,
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let web3 = props.web3;
        let topic = props.topic;

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        Self {
            link,
            topic,
            window,
            storage,
            web3,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
        <div class="chat_window">
            <Display topic=self.topic />
            <Inputs topic=self.topic web3=self.web3/>
        </div>
        }
    }
}
