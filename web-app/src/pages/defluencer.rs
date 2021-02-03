use crate::components::Navbar;
use crate::utils::ens::get_beacon_from_name;
use crate::utils::local_storage::{get_local_beacon, get_local_storage, set_local_beacon};

use wasm_bindgen_futures::spawn_local;

use web_sys::Storage;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use cid::Cid;

pub struct Defluencer {
    ens_name: String,

    beacon_cid: Option<Cid>,

    storage: Option<Storage>,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub ens_name: String,
}

pub enum Msg {
    Beacon(Cid),
}

impl Component for Defluencer {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ens_name = props.ens_name;

        spawn_local(get_beacon_from_name(
            ens_name.clone(),
            link.callback(Msg::Beacon),
        ));

        let window = web_sys::window().expect("Can't get window");
        let storage = get_local_storage(&window);

        let beacon_cid = get_local_beacon(&ens_name, storage.as_ref());

        Self {
            ens_name,
            beacon_cid,
            storage,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Beacon(cid) => self.beacon_update(cid),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="defluencer_page">
            {
                if let Some(cid) = self.beacon_cid {
                    html! {
                        <>
                            <Navbar beacon_cid=cid />
                            <div class="center_text"> {"Channel Page -> W.I.P."} </div>
                        </>
                    }
                } else {
                    html! {
                        <div class="center_text">  {"Loading..."} </div>
                    }
                }
            }
            </div>
        }
    }
}

impl Defluencer {
    /// Receive beacon cid
    fn beacon_update(&mut self, cid: Cid) -> bool {
        #[cfg(debug_assertions)]
        ConsoleService::info("Beacon Update");

        set_local_beacon(&self.ens_name, &cid, self.storage.as_ref());

        self.beacon_cid = Some(cid);

        true
    }
}
