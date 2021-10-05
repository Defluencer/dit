use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use linked_data::mime_type::MimeTyped;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Image {
    pub image_cid: Cid,
    pub ipfs: IpfsService,
    pub image_cb: Callback<Result<MimeTyped>>,

    pub url: String,
}

pub enum Msg {
    Data(Result<MimeTyped>),
}

/// Image from IPFS.
#[derive(Properties, Clone)]
pub struct Props {
    pub image_cid: Cid,
    pub ipfs: IpfsService,
}

impl Component for Image {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { ipfs, image_cid } = props;

        let comp = Self {
            image_cid,
            ipfs,
            image_cb: link.callback(Msg::Data),
            url: String::default(),
        };

        comp.get_image_data();

        comp
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Data(result) => self.on_image_data(result),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.image_cid != self.image_cid {
            self.image_cid = props.image_cid;

            self.get_image_data();
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <img src=self.url.clone() />
        }
    }
}

impl Image {
    fn get_image_data(&self) {
        spawn_local({
            let cb = self.image_cb.clone();
            let ipfs = self.ipfs.clone();
            let cid = self.image_cid;

            async move { cb.emit(ipfs.dag_get(cid, Option::<&str>::None).await) }
        });
    }

    fn on_image_data(&mut self, result: Result<MimeTyped>) -> bool {
        let mime_type = match result {
            Ok(mt) => mt,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        self.url = mime_type.data_url();

        true
    }
}
