use web_sys::{HtmlInputElement, Storage, Window};

use wasm_bindgen::JsCast;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::ChangeData;

use crate::utils::local_storage::{get_local_ipfs_addrs, get_local_storage, set_local_ipfs_addrs};

pub struct Settings {
    link: ComponentLink<Self>,

    window: Window,

    storage: Option<Storage>,
}

pub enum Msg {
    Addrs(ChangeData),
}

impl Component for Settings {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        Self {
            link,
            window,
            storage,
        }
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
            <>
                <h3> { "Settings" } </h3>
                <label for="ipfs_addrs"> { "IPFS API address: " } </label>
                <input type="text" id="ipfs_addrs" name="ipfs_addrs"
                    onchange=self.link.callback(Msg::Addrs)
                    placeholder="IPFS API address" />
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let document = self.window.document().expect("Can't get document");

            let text_area: HtmlInputElement = document
                .get_element_by_id("ipfs_addrs")
                .expect("No element with this Id")
                .dyn_into()
                .expect("Not Input Element");

            if let Some(addrs) = get_local_ipfs_addrs(self.storage.as_ref()).as_ref() {
                text_area.set_value(addrs);
            }
        }
    }
}

impl Settings {
    fn addrs(&mut self, msg: ChangeData) -> bool {
        match msg {
            ChangeData::Value(addrs) => set_local_ipfs_addrs(&addrs, self.storage.as_ref()),
            ChangeData::Select(_) => {}
            ChangeData::Files(_) => {}
        }

        false
    }
}
