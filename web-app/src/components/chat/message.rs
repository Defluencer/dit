use std::rc::Rc;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use yewtil::NeqAssign;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageData {
    pub id: usize,
    pub sender_name: Rc<str>,
    pub message: Rc<str>,
}

impl MessageData {
    fn render(&self) -> Html {
        html! {
            <div class="chat_message">
            //TODO blockies
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
