use crate::components::Navbar;
use crate::utils::{IpfsService, LocalStorage};

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::{Callback, ChangeData};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/* #[derive(PartialEq)]
pub enum NodeType {
    Brave,
    External,
} */

/// Page with app settings and options.
pub struct Settings {
    storage: LocalStorage,
    ipfs: IpfsService,

    peer_id: Option<String>,
    peer_id_cb: Callback<Result<String>>,

    origin: String,

    address: String,
    addrs_cb: Callback<ChangeData>,
    //node_cb: Callback<ChangeData>,
    //node_type: NodeType,
}

pub enum Msg {
    //NodeType(ChangeData),
    Addrs(ChangeData),
    PeerID(Result<String>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ipfs: IpfsService,
    pub storage: LocalStorage,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { storage, ipfs } = props;

        let address = match storage.get_local_ipfs_addrs() {
            Some(addrs) => addrs,
            None => crate::utils::DEFAULT_URI.to_owned(),
        };

        /* let node_type = {
            if address == crate::utils::BRAVE_URI {
                NodeType::Brave
            } else {
                NodeType::External
            }
        }; */

        let mut origin = "*".to_owned();

        if let Some(win) = web_sys::window() {
            match win.location().origin() {
                Ok(org) => origin = org,
                Err(e) => ConsoleService::error(&format!("{:?}", e)),
            }
        }

        let comp = Self {
            storage,
            ipfs,

            peer_id: None,
            peer_id_cb: link.callback(Msg::PeerID),

            origin,

            address,
            addrs_cb: link.callback(Msg::Addrs),
            //node_cb: link.callback(Msg::NodeType),
            //node_type,
        };

        comp.check_ipfs();

        comp
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Addrs(msg) => self.on_addrs(msg),
            //Msg::NodeType(msg) => self.on_node_type(msg),
            Msg::PeerID(msg) => self.on_peer_id(msg),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        //let brave_slct = self.node_type == NodeType::Brave;
        //let ext_slct = self.node_type == NodeType::External;

        html! {
            <>
                <Navbar />
                <ybc::Section>
                    <ybc::Container>
                        {
                            match self.peer_id.as_ref() {
                                Some(peer_id) => self.render_connected(peer_id),
                                None => self.render_not_connected(),
                            }
                        }
                        /* <div class="field">
                            <label class="label"> { "IPFS Node" } </label>
                            <div class="control is-expanded">
                                <div class="select is-fullwidth">
                                    <select id="node_type" onchange=self.node_cb.clone() >
                                        <option selected=brave_slct value="Brave"> { "Brave" } </option>
                                        <option selected=ext_slct value="External"> { "External" } </option>
                                    </select>
                                </div>
                            </div>
                            <p class="help"> { "External nodes can be configured for better performace but Brave browser nodes are more conveniant." } </p>
                        </div> */
                        <div class="field">
                            <label class="label"> { "IPFS API" } </label>
                            <div class="control is-expanded">
                                <input name="ipfs_addrs" value=self.address.clone() onchange=self.addrs_cb.clone() class="input" type="text" />
                            </div>
                            <p class="help"> { "Refresh to apply changes." } </p>
                        </div>
                    </ybc::Container>
                </ybc::Section>
            </>
        }
    }
}

impl Settings {
    fn render_connected(&self, peer_id: &str) -> Html {
        html! {
            <div class="field">
                <label class="label"> { "IPFS Peer ID" } </label>
                <div class="control is-expanded">
                    <input name="ipfs_addrs" value=peer_id.to_owned() class="input is-static" type="text" readonly=true />
                </div>
                <p class="help"> { "A unique string identifing this node on the network." } </p>
            </div>
        }
    }

    fn render_not_connected(&self) -> Html {
        /* let port = if self.node_type == NodeType::Brave {
            "45005"
        } else {
            "5001"
        }; */

        html! {
            <>
                <ybc::Block>
                <span class="icon-text">
                    <span class="icon is-large has-text-danger"><i class="fas fa-exclamation-triangle fa-3x"></i></span>
                    <span class="title"> { "Cannot connect to IPFS" } </span>
                </span>
                </ybc::Block>
                <ybc::Block>
                <h2 class="subtitle">
                    { "Follow the installation guide in the " }
                    <a href="https://docs.ipfs.io/how-to/command-line-quick-start/"> { "IPFS Documentation" } </a>
                    { " or configure your node correctly." }
                </h2>
                </ybc::Block>
                <ybc::Block>
                <ol>
                    <li>
                        <p>{ "Is your IPFS daemon running? Start your daemon with the terminal command below." }</p>
                        <div style="white-space: nowrap;overflow-x: auto;overflow-y: hidden;">
                            <code style="display: block"> { "ipfs daemon --enable-pubsub-experiment --enable-namesys-pubsub" } </code>
                        </div>
                    </li>
                    <li>
                        <p>
                            {"Is your IPFS API configured to allow "}
                            <a href="https://github.com/ipfs-shipyard/ipfs-webui#configure-ipfs-api-cors-headers">
                                {"cross-origin (CORS) requests"}
                            </a>
                            {"? If not, run these terminal commands and restart your daemon."}
                        </p>
                        <div style="white-space: nowrap;overflow-x: auto;overflow-y: hidden;">
                            <code style="display: block"> { r#"ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["POST"]'"# } </code>
                            <code style="display: block"> { format!(r#"ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["https://webui.ipfs.io", "http://127.0.0.1:5001", "{}"]'"#, self.origin) } </code>
                        </div>
                    </li>
                </ol>
                </ybc::Block>
            </>
        }
    }

    fn check_ipfs(&self) {
        spawn_local({
            let cb = self.peer_id_cb.clone();
            let ipfs = self.ipfs.clone();

            async move { cb.emit(ipfs.ipfs_node_id().await) }
        });
    }

    fn on_peer_id(&mut self, response: Result<String>) -> bool {
        let id = match response {
            Ok(id) => id,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return true;
            }
        };

        self.peer_id = Some(id);

        true
    }

    fn on_addrs(&mut self, msg: ChangeData) -> bool {
        let value = match msg {
            ChangeData::Value(value) => value,
            _ => return false,
        };

        if reqwest::Url::parse(&value).is_ok() {
            self.storage.set_local_ipfs_addrs(&value);
        }

        self.address = value;

        false
    }

    /* fn on_node_type(&mut self, msg: ChangeData) -> bool {
        let element = match msg {
            ChangeData::Select(element) => element,
            _ => return false,
        };

        match element.selected_index() {
            0 => {
                self.node_type = NodeType::Brave;
                self.address = crate::utils::BRAVE_URI.to_owned();
                self.storage.set_local_ipfs_addrs(&self.address);
                return true;
            }
            1 => {
                self.node_type = NodeType::External;
                self.address = crate::utils::DEFAULT_URI.to_owned();
                self.storage.set_local_ipfs_addrs(&self.address);
                return true;
            }
            _ => return false,
        }
    } */
}
