use std::collections::VecDeque;
use std::rc::Rc;
use std::str;

use crate::components::{ChatMessage, ChatMessageData};
use crate::utils::bindings::{ipfs_publish, ipfs_subscribe, ipfs_unsubscribe};

use web_sys::{HtmlTextAreaElement, KeyboardEvent, Window};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::InputData;

pub struct ChatWindow {
    link: ComponentLink<Self>,

    topic: String,
    _pubsub_closure: Closure<dyn Fn(String, Vec<u8>)>,
    window: Window,

    temp_msg: Option<String>,
    chat_messages: VecDeque<ChatMessageData>,
    next_id: usize,

    text_area: Option<HtmlTextAreaElement>,
    text_closure: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

pub enum Msg {
    Sent,
    Input(String),
    PubSub((String, Vec<u8>)),
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

        let topic = props.topic;

        let cb = link.callback(Msg::PubSub);
        let _pubsub_closure =
            Closure::wrap(
                Box::new(move |from: String, data: Vec<u8>| cb.emit((from, data)))
                    as Box<dyn Fn(String, Vec<u8>)>,
            );

        ipfs_subscribe(&topic, _pubsub_closure.as_ref().unchecked_ref());

        Self {
            link,
            topic,
            _pubsub_closure,
            window,

            temp_msg: None,
            chat_messages: VecDeque::with_capacity(20),
            next_id: 0,

            text_area: None,
            text_closure: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PubSub((from, data)) => self.on_pubsub_update(from, data),
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

            let cb = self.link.callback(|()| Msg::Sent);

            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                if event.key() == "Enter" {
                    cb.emit(());
                }
            }) as Box<dyn Fn(KeyboardEvent)>);

            text_area
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .expect("Invalid Listener");

            self.text_area = Some(text_area);
            self.text_closure = Some(closure);
        }
    }

    fn destroy(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping Live Chat");

        ipfs_unsubscribe(&self.topic);
    }
}

impl ChatWindow {
    /// Callback when GossipSub receive an update.
    fn on_pubsub_update(&mut self, from: String, data: Vec<u8>) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message");

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        //TODO deserialize to rust type

        /* let content = match str::from_utf8(&data) {
            Ok(data) => data,
            Err(e) => {
                #[cfg(debug_assertions)]
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        }; */

        /* #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Message => {}", content)); */

        /* let message_data = ChatMessageData {
            id: self.next_id,
            sender_name: Rc::from(from),
            message: Rc::from(content),
        }; */

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
            //TODO serialize

            ipfs_publish(&self.topic, msg);

            if let Some(text_area) = self.text_area.as_ref() {
                text_area.set_value("");
            }

            self.temp_msg = None;
        }

        false
    }
}
