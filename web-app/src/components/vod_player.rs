use crate::agents::VODManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

use web_sys::HtmlMediaElement;

use wasm_bindgen::JsCast;

pub struct VODPlayer {
    link: ComponentLink<Self>,

    manager: VODManager,

    video: Option<HtmlMediaElement>,
}

pub enum Msg {
    Add,
    Load,
}

impl Component for VODPlayer {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let manager = VODManager::new();

        Self {
            link,
            manager,
            video: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Add => self.manager.add_source_buffer(),
            Msg::Load => self.manager.load_test_video(),
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <video id="video" poster="../live_like_poster.png" />
                <button onclick=self.link.callback(|_| Msg::Add)>
                    { "Add" }
                </button>
                <button onclick=self.link.callback(|_| Msg::Load)>
                    { "Load" }
                </button>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            //https://medium.com/canal-tech/how-video-streaming-works-on-the-web-an-introduction-7919739f7e1

            let window = web_sys::window().unwrap();

            let document = window.document().unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Document.html#method.get_element_by_id
            let video: HtmlMediaElement = document
                .get_element_by_id("video")
                .unwrap()
                .dyn_into()
                .unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.HtmlVideoElement.html#method.set_src
            video.set_src(&self.manager.url);

            video.set_autoplay(false);
            video.set_controls(true);
            video.set_muted(true);

            self.video = Some(video);
        }
    }
}
