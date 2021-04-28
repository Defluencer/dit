use std::str;

use crate::utils::bindings::ipfs_publish;
use crate::utils::local_storage::{get_cid, get_local_storage, set_cid};
use crate::utils::web3::Web3Service;

use wasm_bindgen_futures::spawn_local;

use web_sys::{HtmlTextAreaElement, KeyboardEvent, Storage, Window};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::{Callback, InputData};

use cid::Cid;

use linked_data::chat::{SignedMessage, UnsignedMessage};
use linked_data::IPLDLink;

use web3::types::Address;

const SIGN_MSG_KEY: &str = "signed_message";

enum State {
    Connect,
    NameOk(String),
    Chatting,
    Error(String),
}

pub struct Inputs {
    link: ComponentLink<Self>,

    topic: String,

    state: State,

    window: Window,
    storage: Option<Storage>,
    web3: Web3Service,

    temp_msg: Option<String>,
    sign_msg: Option<Cid>,

    text_area: Option<HtmlTextAreaElement>,
    text_closure: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

pub enum Msg {
    Input(String),
    Sent,
    Connect,
    Account(Result<Address, web3::Error>),
    Name(Result<String, web3::contract::Error>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub web3: Web3Service,
    pub topic: String,
}

impl Component for Inputs {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let web3 = props.web3;
        let topic = props.topic;

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let sign_msg = get_cid(SIGN_MSG_KEY, storage.as_ref());

        Self {
            link,
            topic,

            state: State::Connect,

            window,
            storage,
            web3,

            temp_msg: None,
            sign_msg,

            text_area: None,
            text_closure: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Input(msg) => self.on_input(msg),
            Msg::Sent => self.send_message(),
            Msg::Connect => self.connect_account(),
            Msg::Account(account) => self.on_account_connected(account),
            Msg::Name(name) => self.on_name_confirmed(name),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let chat = html! {
        <>
        <textarea class="input_text" id="input_text"
        rows=5
        oninput=self.link.callback(|e: InputData| Msg::Input(e.value))
        placeholder="Input text here...">
        </textarea>

        <button class="send_button" onclick=self.link.callback(|_| Msg::Sent)>{ "Send" }</button>
        </>
        };

        let connect = html! {
        <button class="connect_button" onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>
        };

        html! {
        <div class="chat_inputs">
        {
        match self.state {
            State::Chatting => chat,
            State::Connect => connect,
            State::NameOk(name) => html! {
                <form action=self.link.callback(|e: InputData| Msg::Name(e.value))>
                    <label for="name">{"Name"}</label><br/>
                    <input type="text" id="name" name="name" value=name <input/><br/>
                    <input type="submit" value="Submit">
                </form>
                },
            State::Error(e) =>
                html! {
                    < e />
                }
        }
        }
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
}

impl Inputs {
    fn on_input(&mut self, msg: String) -> bool {
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
        let msg = match self.temp_msg.as_ref() {
            Some(msg) => msg,
            None => return false,
        };

        if let Some(text_area) = self.text_area.as_ref() {
            text_area.set_value("");
        }

        self.temp_msg = None;

        let cid = self.sign_msg.expect("Cannot send message with no origin");

        let unsigned = UnsignedMessage {
            message: msg.to_owned(),
            origin: IPLDLink { link: cid },
        };

        ipfs_publish(&self.topic, msg);

        false
    }

    /// Trigger ethereum request accounts.
    fn connect_account(&self) -> bool {
        let cb = self.link.callback(Msg::Account);
        let web3 = self.web3;

        spawn_local(async move { cb.emit(web3.get_eth_accounts().await) });

        false
    }

    /// Callback with response of request accounts.
    fn on_account_connected(&mut self, response: Result<Address, web3::Error>) -> bool {
        match response {
            Ok(address) => {
                let cb = self.link.callback(Msg::Name);
                let web3 = self.web3;

                spawn_local(async move { cb.emit(web3.get_name(address).await) });

                //TODO get peer id

                //TODO sign msg

                //TODO add to IPFS

                //TODO add to storage
            }
            Err(e) => {
                //TODO display error
            }
        }

        true
    }

    fn on_name_confirmed(&mut self, response: Result<String, web3::contract::Error>) -> bool {
        let name = response.unwrap_or_default();

        self.state = State::NameOk(name);

        //TODO get peer id and name then sign msg

        true
    }
}
