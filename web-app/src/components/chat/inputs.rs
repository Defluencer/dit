use std::rc::Rc;
use std::str;

use crate::utils::{IpfsService, LocalStorage, Web3Service};

use wasm_bindgen_futures::spawn_local;

use web_sys::{HtmlTextAreaElement, KeyboardEvent};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::InputData;

use cid::Cid;

use linked_data::beacon::Beacon;
use linked_data::chat::{ChatId, Message, MessageType, UnsignedMessage};
use linked_data::signature::SignedMessage;

use web3::types::Address;

const SIGN_MSG_KEY: &str = "signed_message";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum DisplayState {
    /// Before anything, ask user to connect account.
    Connect,
    /// Has use reverse resolution to get a name from an address.
    NameOk(String),
    /// The signature is ready and can be use for messages.
    Chatting,
}

pub struct Inputs {
    props: Props,
    link: ComponentLink<Self>,

    state: DisplayState,

    temp_msg: Option<String>,

    address: Option<Address>,
    peer_id: Option<String>,
    name: Option<String>,
    sign_msg_content: Option<ChatId>,
    sign_msg_cid: Option<Cid>,

    text_area: Option<HtmlTextAreaElement>,
    text_closure: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

pub enum Msg {
    Set(String),
    Enter,
    Connect,
    PeerID(Result<String>),
    Account(Result<Address>),
    AccountName(Result<String>),
    SetName(String),
    SubmitName,
    Signed(Result<[u8; 65]>),
    Minted(Result<Cid>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub web3: Web3Service,
    pub storage: LocalStorage,
    pub beacon: Rc<Beacon>,
}

impl Component for Inputs {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let (sign_msg_cid, state) = match props.storage.get_cid(SIGN_MSG_KEY) {
            Some(cid) => (Some(cid), DisplayState::Chatting),
            None => (None, DisplayState::Connect),
        };

        //TODO should verify that the node has not been garbage collected in between session.

        Self {
            props,
            link,

            state,

            temp_msg: None,

            address: None,
            peer_id: None,
            name: None,
            sign_msg_content: None,
            sign_msg_cid,

            text_area: None,
            text_closure: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Set(msg) => self.on_chat_input(msg),
            Msg::Enter => self.send_message(),
            Msg::Connect => self.connect_account(),
            Msg::PeerID(res) => self.on_peer_id(res),
            Msg::Account(res) => self.on_account_connected(res),
            Msg::AccountName(res) => self.on_account_name(res),
            Msg::SetName(name) => self.on_name_input(name),
            Msg::SubmitName => self.on_name_submit(),
            Msg::Signed(res) => self.on_signature(res),
            Msg::Minted(res) => self.on_sign_msg_minted(res),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.beacon != self.props.beacon {
            self.props = props;

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        let content = match &self.state {
            DisplayState::Chatting => {
                html! {
                <div>
                    <textarea class="input_text" id="input_text"
                    rows=5
                    oninput=self.link.callback(|e: InputData| Msg::Set(e.value))
                    placeholder="Input text here...">
                    </textarea>
                    <button class="send_button" onclick=self.link.callback(|_| Msg::Enter)>{ "Send" }</button>
                </div> }
            }
            DisplayState::Connect => {
                html! { <button class="connect_button" onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button> }
            }
            DisplayState::NameOk(name) => {
                html! {
                <div class="submit_name">
                    <label class="name_label">{ "Name" }<input placeholder=name.clone() oninput=self.link.callback(|e: InputData|  Msg::SetName(e.value)) /></label>
                    <button class="submit_button" onclick=self.link.callback_once(|_|  Msg::SubmitName)>{ "Confirm" }</button>
                </div> }
            }
        };

        html! {
            <div class="chat_inputs">
            { content }
            </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.text_area.is_some() {
            return;
        }

        if let DisplayState::Chatting = self.state {
            let window = match web_sys::window() {
                Some(window) => window,
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Window Object");
                    return;
                }
            };

            let document = match window.document() {
                Some(document) => document,
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Document Object");
                    return;
                }
            };

            let element = match document.get_element_by_id("input_text") {
                Some(document) => document,
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Element by Id");
                    return;
                }
            };

            let text_area: HtmlTextAreaElement = match element.dyn_into() {
                Ok(document) => document,
                Err(e) => {
                    ConsoleService::error(&format!("{:#?}", e));
                    return;
                }
            };

            let cb = self.link.callback(|()| Msg::Enter);

            let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
                if event.key() == "Enter" {
                    cb.emit(());
                }
            }) as Box<dyn Fn(KeyboardEvent)>);

            if let Err(e) = text_area
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            {
                ConsoleService::error(&format!("{:#?}", e));
            }

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
        let message = match self.temp_msg.take() {
            Some(msg) => msg,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Message");
                return false;
            }
        };

        if let Some(text_area) = self.text_area.as_ref() {
            text_area.set_value("");
        }

        let cid = match self.sign_msg_cid {
            Some(cid) => cid,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Signed Message CID");
                return false;
            }
        };

        let msg = Message {
            msg_type: MessageType::Unsigned(UnsignedMessage { message }),

            origin: cid.into(),
        };

        let json_string = match serde_json::to_string(&msg) {
            Ok(json_string) => json_string,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                return false;
            }
        };

        let client = self.props.ipfs.clone();
        let topic = self.props.beacon.topics.live_chat.clone();

        #[cfg(debug_assertions)]
        ConsoleService::info("Publish Message");

        spawn_local(async move {
            if let Err(e) = client.pubsub_pub(topic, json_string).await {
                ConsoleService::error(&format!("{:#?}", e));
            }
        });

        false
    }

    /// Trigger ethereum request accounts.
    fn connect_account(&self) -> bool {
        let cb = self.link.callback(Msg::Account);
        let web3 = self.props.web3.clone();

        #[cfg(debug_assertions)]
        ConsoleService::info("Get Address");

        spawn_local(async move { cb.emit(web3.get_eth_accounts().await) });

        let cb = self.link.callback(Msg::PeerID);
        let client = self.props.ipfs.clone();

        #[cfg(debug_assertions)]
        ConsoleService::info("Get Peer ID");

        spawn_local(async move { cb.emit(client.ipfs_node_id().await) });

        false
    }

    /// Callback with response of request accounts.
    fn on_account_connected(&mut self, response: Result<Address>) -> bool {
        let address = match response {
            Ok(address) => address,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                self.state = DisplayState::Connect;
                return true;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Address => {}", &address.to_string()));

        self.address = Some(address);

        let cb = self.link.callback(Msg::AccountName);
        let web3 = self.props.web3.clone();

        spawn_local(async move { cb.emit(web3.get_name(address).await) });

        false
    }

    fn on_peer_id(&mut self, response: Result<String>) -> bool {
        let id = match response {
            Ok(id) => id,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                self.state = DisplayState::Connect;
                return true;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Peer ID => {}", &id));

        self.peer_id = Some(id);

        false
    }

    fn on_account_name(&mut self, response: Result<String>) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Name Revolved");

        let name = match response {
            Ok(string) => string,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));

                String::new()
            }
        };

        self.state = DisplayState::NameOk(name);

        true
    }

    fn on_name_input(&mut self, name: String) -> bool {
        self.name = Some(name);

        false
    }

    fn on_name_submit(&mut self) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Name Submitted");

        let address = match self.address {
            Some(addrs) => addrs,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Address");
                return false;
            }
        };

        let peer = match self.peer_id.take() {
            Some(peer) => peer,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Peer Id");
                return false;
            }
        };

        let name = match self.name.take() {
            Some(name) => name,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Name");
                return false;
            }
        };

        let cb = self.link.callback_once(Msg::Signed);
        let web3 = self.props.web3.clone();
        let data = ChatId { name, peer };

        self.sign_msg_content = Some(data.clone());

        spawn_local(async move { cb.emit(web3.eth_sign(address, data).await) });

        false
    }

    fn on_signature(&mut self, response: Result<[u8; 65]>) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Signature Received");

        let signature = match response {
            Ok(sig) => sig.to_vec(),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                self.state = DisplayState::Connect;
                return true;
            }
        };

        let address = match self.address.take() {
            Some(addrs) => addrs.to_fixed_bytes(),
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Address");
                return false;
            }
        };

        let data = match self.sign_msg_content.take() {
            Some(data) => data,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Signed Message Content");
                return false;
            }
        };

        let signed_msg = SignedMessage {
            address,
            data,
            signature,
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Verifiable => {}", &signed_msg.verify()));

        let cb = self.link.callback_once(Msg::Minted);
        let client = self.props.ipfs.clone();

        spawn_local(async move { cb.emit(client.dag_put(&signed_msg).await) });

        false
    }

    fn on_sign_msg_minted(&mut self, response: Result<Cid>) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Signed Message Minted");

        let cid = match response {
            Ok(cid) => cid,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                self.state = DisplayState::Connect;
                return true;
            }
        };

        self.props.storage.set_cid(SIGN_MSG_KEY, &cid);

        self.sign_msg_cid = Some(cid);
        self.state = DisplayState::Chatting;

        true
    }
}
