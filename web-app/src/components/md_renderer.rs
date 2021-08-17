use crate::utils::render_markdown;
use crate::utils::IpfsService;

use wasm_bindgen_futures::spawn_local;

use yew::prelude::{Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Markdown renderer.
pub struct Markdown {
    text: String,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,
    pub markdown_cid: Cid,
}

pub enum Msg {
    File(Result<Vec<u8>>),
}

impl Component for Markdown {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        spawn_local({
            let cb = link.callback_once(Msg::File);
            let ipfs = props.ipfs.clone();
            let cid = props.markdown_cid;

            async move { cb.emit(ipfs.cid_cat(cid).await) }
        });

        Self {
            text: String::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::File(result) => self.update_file(result),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        render_markdown(&self.text)
    }
}

impl Markdown {
    fn update_file(&mut self, response: Result<Vec<u8>>) -> bool {
        let data = match response {
            Ok(data) => data,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                return false;
            }
        };

        let string = match String::from_utf8(data) {
            Ok(slice) => slice,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                return false;
            }
        };

        self.text = string;

        true
    }
}
