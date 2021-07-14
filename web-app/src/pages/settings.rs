use crate::components::Navbar;
use crate::utils::LocalStorage;

use web_sys::HtmlInputElement;

use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::ChangeData;

pub struct Settings {
    link: ComponentLink<Self>,

    storage: LocalStorage,
}

pub enum Msg {
    Addrs(ChangeData),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub storage: LocalStorage, // From app.
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { storage } = props;

        Self { link, storage }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Addrs(msg) => self.addrs(msg),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="settings_page">
                <Navbar />
                <div class="settings">
                    <h3> { "Settings" } </h3>
                    <div>
                        <label for="ipfs_addrs"> { "IPFS API address: " } </label>
                        <input type="text" id="ipfs_addrs" name="ipfs_addrs"
                            onchange=self.link.callback(Msg::Addrs)
                            placeholder="IPFS API address" />
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let window = match web_sys::window() {
                Some(window) => window,
                None => return,
            };

            let document = match window.document() {
                Some(document) => document,
                None => return,
            };

            let element = match document.get_element_by_id("ipfs_addrs") {
                Some(document) => document,
                None => return,
            };

            let text_area: HtmlInputElement = match element.dyn_into() {
                Ok(document) => document,
                Err(e) => {
                    ConsoleService::error(&format!("{:#?}", e));
                    return;
                }
            };

            if let Some(addrs) = self.storage.get_local_ipfs_addrs() {
                text_area.set_value(&addrs);
            }
        }
    }
}

impl Settings {
    fn addrs(&mut self, msg: ChangeData) -> bool {
        match msg {
            ChangeData::Value(addrs) => self.storage.set_local_ipfs_addrs(&addrs),
            ChangeData::Select(_) => {}
            ChangeData::Files(_) => {}
        }

        false
    }
}
