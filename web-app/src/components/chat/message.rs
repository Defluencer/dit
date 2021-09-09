use std::rc::Rc;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use cid::multibase::Base;

#[derive(Clone)]
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
            <ybc::Message>
                <ybc::MessageHeader>
                    <ybc::Image size=ybc::ImageSize::IsSquare >
                        <img src=self.img_data.to_string() height="32" width="32" />
                    </ybc::Image>
                    <h3>{ &self.sender_name }</h3>
                </ybc::MessageHeader>
                <ybc::MessageBody>
                    { &self.message }
                </ybc::MessageBody>
            </ybc::Message>
        }
    }
}

#[derive(Clone, Properties)]
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
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if !Rc::ptr_eq(&self.message_data.img_data, &props.message_data.img_data)
            || !Rc::ptr_eq(
                &self.message_data.sender_name,
                &props.message_data.sender_name,
            )
            || !Rc::ptr_eq(&self.message_data.message, &props.message_data.message)
        {
            *self = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        self.message_data.render()
    }
}
