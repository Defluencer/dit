use crate::components::Navbar;
use crate::utils::LocalStorage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::{Callback, ChangeData};

#[derive(PartialEq)]
pub enum NodeType {
    Brave,
    External,
}

/// Page with app settings and options.
pub struct Settings {
    address: String,
    addrs_cb: Callback<ChangeData>,

    node_cb: Callback<ChangeData>,
    node_type: NodeType,

    storage: LocalStorage,
}

pub enum Msg {
    NodeType(ChangeData),
    Addrs(ChangeData),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub storage: LocalStorage,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { storage } = props;

        let address = match storage.get_local_ipfs_addrs() {
            Some(addrs) => addrs,
            None => crate::utils::DEFAULT_URI.to_owned(),
        };

        let node_type = {
            if address == crate::utils::BRAVE_URI {
                NodeType::Brave
            } else {
                NodeType::External
            }
        };

        Self {
            address,
            addrs_cb: link.callback(Msg::Addrs),

            node_cb: link.callback(Msg::NodeType),
            node_type,

            storage,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Addrs(msg) => self.addrs(msg),
            Msg::NodeType(msg) => self.node_type(msg),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let brave_slt = self.node_type == NodeType::Brave;
        let ext_slt = self.node_type == NodeType::External;

        html! {
            <>
                <Navbar />
                <ybc::Section>
                    <ybc::Container>
                        <div class="field">
                            <label class="label"> { "IPFS Node" } </label>
                            <div class="control is-expanded">
                                <div class="select is-fullwidth">
                                    <select id="node_type" onchange=self.node_cb.clone() >
                                        <option selected=brave_slt value="Brave"> { "Brave" } </option>
                                        <option selected=ext_slt value="External"> { "External" } </option>
                                    </select>
                                </div>
                            </div>
                            <p class="help"> { "External nodes can be configured for better performace but Brave browser nodes are more conveniant." } </p>
                        </div>
                        <div class="field">
                            <label class="label"> { "IPFS API" } </label>
                            <div class="control is-expanded">
                                <input name="ipfs_addrs" value=self.address.clone() onchange=self.addrs_cb.clone() class="input" type="text" readonly=brave_slt />
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
    fn node_type(&mut self, msg: ChangeData) -> bool {
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
    }

    fn addrs(&mut self, msg: ChangeData) -> bool {
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
}
