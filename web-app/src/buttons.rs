use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
use yew::services::ConsoleService;
use yew::MouseEvent;

use web_sys::HtmlMediaElement;

use wasm_bindgen::JsCast;

pub struct PlayButton {
    link: ComponentLink<Self>,

    video: Option<HtmlMediaElement>,
}

pub enum Msg {
    Click,
}

impl Component for PlayButton {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, video: None }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                if self.video.is_none() {
                    let document = web_sys::window().unwrap().document().unwrap();

                    let video = document.get_element_by_id("video").unwrap();

                    self.video = Some(video.unchecked_into());
                }

                let _ = self.video.as_ref().unwrap().play();
            }
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let click_callback = self.link.callback(|_: MouseEvent| Msg::Click);

        html! {
                <button onclick=click_callback>
                { "Play" }
                </button>
        }
    }
}
