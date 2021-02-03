use std::collections::VecDeque;
use std::rc::Rc;

use crate::agents::LiveChatManager;
use crate::components::{ChatMessage, ChatMessageData};

use web_sys::{HtmlTextAreaElement, KeyboardEvent, Window};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::{Callback, InputData};

pub struct ChatWindow {
    link: ComponentLink<Self>,

    temp_msg: Option<String>,

    chat_messages: VecDeque<ChatMessageData>,

    next_id: usize,

    manager: LiveChatManager,

    window: Window,

    text_area: Option<HtmlTextAreaElement>,
}

pub enum Msg {
    Received((String, String)),
    Sent,
    Input(String),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub topic: String,
}

impl Component for ChatWindow {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let window = web_sys::window().expect("Can't get window");

        let cb = link.callback(Msg::Received);

        let manager = LiveChatManager::new(props.topic, cb);

        Self {
            link,
            temp_msg: None,
            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,
            manager,
            text_area: None,
            window,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Received(message) => self.msg_received(message),
            Msg::Input(msg) => self.input(msg),
            Msg::Sent => self.send_message(),
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
                        for self.chat_messages.iter().map(|cm| html! {
                            <ChatMessage key=cm.id.to_string() message_data=cm />
                        })
                    }
                </div>

                <textarea class="input_text" id="input_text"
                    rows=5
                    oninput=self.link.callback(|e: InputData| Msg::Input(e.value))
                    placeholder="Input text here...">
                </textarea>

                <button class="send_button" onclick=self.link.callback(|_| Msg::Sent)>{ "Send" }</button>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let document = self.window.document().expect("Can't get document");

            let text_area: HtmlTextAreaElement = document
                .get_element_by_id("input_text")
                .expect("No element with this Id")
                .dyn_into()
                .expect("Not Text Area Element");

            on_enter(&text_area, self.link.callback(|()| Msg::Sent));

            self.text_area = Some(text_area);
        }
    }
}

impl ChatWindow {
    fn msg_received(&mut self, message: (String, String)) -> bool {
        let (sender, content) = message;

        let message_data = ChatMessageData {
            id: self.next_id,
            sender_name: Rc::from(sender),
            message: Rc::from(content),
        };

        self.chat_messages.push_back(message_data);

        if self.chat_messages.len() >= 10 {
            self.chat_messages.pop_front();
        }

        self.next_id += 1;

        true
    }

    fn input(&mut self, msg: String) -> bool {
        if msg == "\n" {
            if let Some(text_area) = self.text_area.as_ref() {
                text_area.set_value("");
            }

            return false;
        }

        self.temp_msg = Some(msg);

        false
    }

    fn send_message(&mut self) -> bool {
        if let Some(msg) = self.temp_msg.as_ref() {
            self.manager.send_chat(msg);

            if let Some(text_area) = self.text_area.as_ref() {
                text_area.set_value("");
            }

            self.temp_msg = None;
        }

        false
    }
}

fn on_enter(text_area: &HtmlTextAreaElement, cb: Callback<()>) {
    let closure = move |event: KeyboardEvent| {
        if event.key() == "Enter" {
            cb.emit(());
        }
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn(KeyboardEvent)>);

    if let Err(e) = text_area
        .add_event_listener_with_callback("keydown", callback.into_js_value().unchecked_ref())
    {
        ConsoleService::error(&format!("{:?}", e));
    }
}
