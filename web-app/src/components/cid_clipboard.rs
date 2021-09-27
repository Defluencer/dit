use web_sys::{window, Clipboard};

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;
use yew::{Callback, MouseEvent};

use cid::Cid;

/// Copy CID Button.
pub struct CidClipboard {
    cid: Cid,
    cb: Callback<MouseEvent>,
    clipboard: Option<Clipboard>,
}

pub enum Msg {
    Clip,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub cid: Cid,
}

impl Component for CidClipboard {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props { cid } = props;

        let clipboard = get_clipboard();

        Self {
            cid,
            cb: link.callback(|_event: MouseEvent| Msg::Clip),
            clipboard,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clip => {
                //TODO copy to cilp board
                if let Some(clipboard) = &self.clipboard {
                    let _pro = clipboard.write_text(&self.cid.to_string());
                }
            }
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <ybc::Button classes=classes!("is-small", "is-outlined", "is-primary") onclick=self.cb.clone() >
                <span class="icon"><i class="fas fa-copy"></i></span>
                <span> { "CID" } </span>
            </ybc::Button>
        }
    }
}

fn get_clipboard() -> Option<Clipboard> {
    let window = match window() {
        Some(window) => window,
        None => {
            #[cfg(debug_assertions)]
            ConsoleService::error("No Window Object");
            return None;
        }
    };

    let navigator = window.navigator();

    navigator.clipboard()
}
