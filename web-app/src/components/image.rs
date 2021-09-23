use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::Callback;

use cid::multibase::Base;
use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Image {
    pub image_cid: Cid,
    pub ipfs: IpfsService,
    pub image_cb: Callback<Result<Vec<u8>>>,

    pub url: String,
}

pub enum Msg {
    Data(Result<Vec<u8>>),
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

            return true;
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <img  src=self.url.clone() />
        }
    }
}

impl Image {
    fn get_image_data(&self) {
        spawn_local({
            let cb = self.image_cb.clone();
            let ipfs = self.ipfs.clone();
            let cid = self.image_cid;

            async move { cb.emit(ipfs.cid_cat(cid).await) }
        });
    }

    fn on_image_data(&mut self, result: Result<Vec<u8>>) -> bool {
        let data = match result {
            Ok(data) => data,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        let mut encoded = Base::Base64.encode(data);

        encoded.insert_str(0, "data:image/jpg;base64,");

        self.url = encoded;

        true
    }
}
