use std::collections::VecDeque;
use std::rc::Rc;

use crate::agents::{load_live_chat, send_chat, unload_live_chat};
use crate::components::{ChatMessage, ChatMessageData};

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::InputData;

pub struct ChatWindow {
    link: ComponentLink<Self>,

    temp_msg: String,

    chat_messages: VecDeque<ChatMessageData>,

    next_id: usize,
}

pub enum Msg {
    Received(String),
    Sent,
    Input(String),
}

impl Component for ChatWindow {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb = link.callback(Msg::Received);

        load_live_chat(cb);

        Self {
            link,
            temp_msg: "No message yet.".into(),
            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Received(message) => {
                let message_data = ChatMessageData {
                    id: self.next_id,
                    sender_name: Rc::from("Placeholder Name"),
                    message: Rc::from(message),
                };

                self.chat_messages.push_back(message_data);

                if self.chat_messages.len() >= 20 {
                    self.chat_messages.pop_front();
                }

                self.next_id += 1;

                true
            }
            Msg::Input(msg) => {
                self.temp_msg = msg;

                false
            }
            Msg::Sent => {
                send_chat(self.temp_msg.clone());

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
                <div class="chats">
                    {
                        for self.chat_messages.iter().rev().map(|cm| html! {
                            <ChatMessage key=cm.id.to_string() message_data=cm />
                        })
                    }
                </div>

                <textarea class="input_text"
                    rows=5
                    oninput=self.link.callback(|e: InputData| Msg::Input(e.value))
                    placeholder="Input text here...">
                </textarea>

                <button class="send_button" onclick=self.link.callback(|_| Msg::Sent)>{ "Send" }</button>
            </div>
        }
    }

    fn destroy(&mut self) {
        unload_live_chat();
    }
}
