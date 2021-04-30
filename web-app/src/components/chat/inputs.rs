use std::rc::Rc;
use std::str;

use crate::utils::ipfs::IpfsService;
use crate::utils::local_storage::{get_cid, get_local_storage, set_cid};
use crate::utils::web3::Web3Service;

use wasm_bindgen_futures::spawn_local;

use web_sys::{HtmlTextAreaElement, KeyboardEvent, Storage, Window};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::InputData;

use cid::Cid;

use linked_data::chat::{Content, SignedMessage, UnsignedMessage};
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

    ipfs: IpfsService,
    topic: Rc<str>,
    state: State,

    window: Window,
    storage: Option<Storage>,
    web3: Web3Service,

    temp_msg: Option<String>,

    address: Option<Address>,
    peer_id: Option<String>,
    name: Option<String>,
    sign_msg: Option<Cid>,

    text_area: Option<HtmlTextAreaElement>,
    text_closure: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

pub enum Msg {
    SetMsg(String),
    SendMsg,
    Connect,
    Account(Result<Address, web3::Error>),
    AccountName(Result<String, web3::contract::Error>),
    SetName(String),
    SubmitName,
    Signed(Result<[u8; 65], web3::Error>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub web3: Web3Service,
    pub topic: Rc<str>,
}

impl Component for Inputs {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, web3, topic } = props;

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let (sign_msg, state) = match get_cid(SIGN_MSG_KEY, storage.as_ref()) {
            Some(msg) => (Some(msg), State::Chatting),
            None => (None, State::Connect),
        };

        Self {
            link,

            ipfs,
            topic,
            state,

            window,
            storage,
            web3,

            temp_msg: None,

            sign_msg,
            address: None,
            peer_id: None,
            name: None,

            text_area: None,
            text_closure: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetMsg(msg) => self.on_chat_input(msg),
            Msg::SendMsg => self.send_message(),
            Msg::Connect => self.connect_account(),
            Msg::Account(res) => self.on_account_connected(res),
            Msg::AccountName(res) => self.on_account_name(res),
            Msg::SetName(name) => self.on_name_input(name),
            Msg::SubmitName => self.on_name_submit(),
            Msg::Signed(res) => self.on_signature(res),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let content = match &self.state {
            State::Chatting => {
                html! {
                <div>
                    <textarea class="input_text" id="input_text"
                    rows=5
                    oninput=self.link.callback(|e: InputData| Msg::SetMsg(e.value))
                    placeholder="Input text here...">
                    </textarea>
                    <button class="send_button" onclick=self.link.callback(|_| Msg::SendMsg)>{ "Send" }</button>
                </div> }
            }
            State::Connect => {
                html! { <button class="connect_button" onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button> }
            }
            State::NameOk(name) => {
                html! {
                <form id="submit_name">
                    <label class="name_label"><input placeholder=name oninput=self.link.callback(|e: InputData|  Msg::SetName(e.value)) /></label>
                    <button class="submit_button" onclick=self.link.callback(|_|  Msg::SubmitName)>{ "Confirm" }</button>
                </form> }
            }
            State::Error(e) => {
                html! { <div> { e } </div> }
            }
        };

        html! {
            <div class="chat_inputs">
            { content }
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

            let cb = self.link.callback(|()| Msg::SendMsg);

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
    fn on_chat_input(&mut self, msg: String) -> bool {
        if msg == "\n" {
            if let Some(text_area) = self.text_area.as_ref() {
                text_area.set_value("");
            }

            return false;
        }

        self.temp_msg = Some(msg);

        false
    }

    /// Send chat message via gossipsub.
    fn send_message(&mut self) -> bool {
        let msg = match self.temp_msg.as_mut() {
            Some(msg) => msg,
            None => return false,
        };

        if let Some(text_area) = self.text_area.as_ref() {
            text_area.set_value("");
        }

        let cid = self.sign_msg.expect("Cannot send message with no origin");

        let msg = serde_json::to_string(&UnsignedMessage {
            message: msg.clone(),
            origin: IPLDLink { link: cid },
        })
        .expect("Cannot serialize");

        self.temp_msg = None;

        //TODO ipfs_publish(&self.topic, &msg);

        false
    }

    /// Trigger ethereum request accounts.
    fn connect_account(&self) -> bool {
        let cb = self.link.callback(Msg::Account);
        let web3 = self.web3.clone();

        spawn_local(async move { cb.emit(web3.get_eth_accounts().await) });

        false
    }

    /// Callback with response of request accounts.
    fn on_account_connected(&mut self, response: Result<Address, web3::Error>) -> bool {
        match response {
            Ok(address) => {
                #[cfg(debug_assertions)]
                ConsoleService::info(&format!("Address => {}", &address.to_string()));

                self.address = Some(address);

                let cb = self.link.callback(Msg::AccountName);
                let web3 = self.web3.clone();
                spawn_local(async move { cb.emit(web3.get_name(address).await) });
            }
            Err(e) => self.state = State::Error(e.to_string()),
        }

        false
    }

    fn on_account_name(&mut self, response: Result<String, web3::contract::Error>) -> bool {
        let name = response.unwrap_or_default();

        self.state = State::NameOk(name);

        true
    }

    fn on_name_input(&mut self, name: String) -> bool {
        self.name = Some(name);

        false
    }

    fn on_name_submit(&mut self) -> bool {
        let address = self.address.take().expect("Invalid Address");
        let peer_id = self.peer_id.take().expect("Invalid Peer Id");
        let name = self.name.take().expect("Invalid Name");

        let cb = self.link.callback_once(Msg::Signed);
        let web3 = self.web3.clone();
        let data = Content { peer_id, name };

        spawn_local(async move { cb.emit(web3.eth_sign(address, data).await) });

        false
    }

    fn on_signature(&mut self, reponse: Result<[u8; 65], web3::Error>) -> bool {
        let signature = match reponse {
            Ok(sig) => sig.to_vec(),
            Err(e) => {
                self.state = State::Error(e.to_string());
                return true;
            }
        };

        let address = self
            .address
            .take()
            .expect("Invalid Address")
            .to_fixed_bytes();

        //TODO get peer id somehow
        let peer_id = self.peer_id.take().expect("Invalid Peer Id");

        let name = self.name.take().expect("Invalid Name");

        let data = Content { peer_id, name };

        let signed_msg = SignedMessage {
            address,
            data,
            signature,
        };

        //TODO add to IPFS

        /* set_cid(SIGN_MSG_KEY, &cid, self.storage.as_ref());

        self.state = State::Chatting; */

        true
    }
}
