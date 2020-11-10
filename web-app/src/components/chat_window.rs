use crate::agents::ChatManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::worker::Bridge;
use yew::Bridged;

pub struct ChatWindow {
    link: ComponentLink<Self>,

    manager: Box<dyn Bridge<ChatManager>>,

    message: String,
}

pub enum Msg {
    Text(String),
    Clicked,
}

impl Component for ChatWindow {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Msg::Text);

        Self {
            link,
            manager: ChatManager::bridge(cb),
            message: "No message yet.".into(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Text(message) => {
                self.message = message;

                true
            }
            Msg::Clicked => {
                self.manager.send(String::from("Message sent from UI"));

                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="chat_window">
                { &self.message }
                <button onclick=self.link.callback(|_| Msg::Clicked)>
                    { "Send Message" }
                </button>
            </div>
        }
    }
}
