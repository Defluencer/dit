use std::rc::Rc;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use yewtil::NeqAssign;

use cid::multibase::Base;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageData {
    pub id: usize,
    img_data: Rc<str>,
    sender_name: Rc<str>,
    message: Rc<str>,
}

impl MessageData {
    pub fn new(id: usize, img_data: &[u8], name: &str, message: &str) -> Self {
        let base = Base::Base64;
        let encoded = base.encode(img_data);
        let url = format!("data:image/png;base64,{}", encoded);

        Self {
            id,
            img_data: Rc::from(url),
            sender_name: Rc::from(name),
            message: Rc::from(message),
        }
    }

    fn render(&self) -> Html {
        html! {
            <div class="chat_message">
                <img src=self.img_data height="32" width="32" />
                <h3>{ &self.sender_name }</h3>
                <p>{ &self.message }</p>
            </div>
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct UIMessage {
    pub message_data: MessageData,
}

impl Component for UIMessage {
    type Message = ();
    type Properties = Self;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.neq_assign(props)
    }

    fn view(&self) -> Html {
        self.message_data.render()
    }
}
