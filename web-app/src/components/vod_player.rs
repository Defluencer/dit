//use crate::bindings;
//use crate::vod_manager::VODManager;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};

pub struct VODPlayer {
    //vod: VODManager,
}

impl Component for VODPlayer {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        //let vod = VODManager::new();

        //vod.register_callback();

        Self { /* vod */ }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                /* <video id="video" inline=true muted=true controls=true poster="/live_like_poster.png">
                </video> */
                {"W.I.P."}
            </>
        }
    }

    /* fn rendered(&mut self, first_render: bool) {
        if first_render {
            bindings::attach_media(); // Must be called after <video> is added
        }
    } */

    /* fn destroy(&mut self) {
        bindings::destroy();
    } */
}
