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
    pub image_cb: Callback<Result<String>>,

    pub url: String,
}

pub enum Msg {
    Data(Result<String>),
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
            Msg::Data(result) => self.on_data_url(result),
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

            async move {
                let mime_type = match ipfs
                    .dag_get::<_, MimeTyped>(cid, Option::<&str>::None)
                    .await
                {
                    Ok(mt) => mt,
                    Err(e) => {
                        cb.emit(Err(e));
                        return;
                    }
                };

                let data = match ipfs.cid_cat(mime_type.data.link).await {
                    Ok(mt) => mt,
                    Err(e) => {
                        cb.emit(Err(e));
                        return;
                    }
                };

                cb.emit(Ok(mime_type.data_url(&data)))
            }
        });
    }

    fn on_data_url(&mut self, result: Result<String>) -> bool {
        let data_url = match result {
            Ok(url) => url,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return false;
            }
        };

        self.url = data_url;

        true
    }
}
