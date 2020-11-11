use std::rc::Rc;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use yewtil::NeqAssign;

//use cid::Cid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatMessageData {
    pub id: usize,
    //pub sender_cid: Cid,
    pub sender_name: Rc<str>,
    pub message: Rc<str>,
}

impl ChatMessageData {
    fn render(&self) -> Html {
        html! {
            <div class="chat_message">
                <h1>{ &self.sender_name }</h1>
                <p>{ &self.message }</p>
            </div>
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct ChatMessage {
    message_data: ChatMessageData,
}

impl Component for ChatMessage {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <div class="chat_message_component">
                { self.message_data.render() }
            </div>
        }
    }
}

impl ChatMessage {
    pub fn render(self) -> Html {
        html! {
            <ChatMessage key=self.message_data.id.to_string() message_data=self.message_data />
        }
    }
}
