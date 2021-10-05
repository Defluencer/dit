use std::rc::Rc;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use linked_data::mime_type::MimeTyped;

#[derive(Clone)]
pub struct MessageData {
    pub id: usize,
    img_data: Rc<str>,
    sender_name: Rc<str>,
    message: Rc<str>,
}

impl MessageData {
    pub fn new(id: usize, img_data: &[u8], name: &str, message: &str) -> Self {
        let url = MimeTyped::new("image/png", img_data).data_url();

        Self {
            id,
            img_data: Rc::from(url),
            sender_name: Rc::from(name),
            message: Rc::from(message),
        }
    }

    fn render(&self) -> Html {
        html! {
            <article class="message is-small" style="overflow-wrap: break-word" >
                <ybc::MessageHeader>
                    <ybc::Image size=ybc::ImageSize::IsSquare >
                        <img src=self.img_data.to_string() height="32" width="32" />
                    </ybc::Image>
                    <h3>{ &self.sender_name }</h3>
                </ybc::MessageHeader>
                <ybc::MessageBody>
                    { &self.message }
                </ybc::MessageBody>
            </article>
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
