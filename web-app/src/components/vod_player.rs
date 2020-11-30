use crate::agents::load_video;
//use crate::bindings;

//use wasm_bindgen::closure::Closure;
//use wasm_bindgen::JsCast;

//use web_sys::HtmlMediaElement;

use yew::prelude::{html, Component, ComponentLink, Html, ShouldRender};
//use yew::services::ConsoleService;

pub struct VODPlayer {
    //video: Option<HtmlMediaElement>,
}

impl Component for VODPlayer {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            
            //video: None,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <video id="video" autoplay=true controls=true muted=true poster="../live_like_poster.png" />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            //bindings::load_video("bafyreic6hsipoya2rpn3eankfplts6yvxevuztakn2uof4flnbt2ipwlue");

            load_video();

            //https://medium.com/canal-tech/how-video-streaming-works-on-the-web-an-introduction-7919739f7e1

            //let window = web_sys::window().unwrap();

            //let document = window.document().unwrap();

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Document.html#method.get_element_by_id
            /* let video: HtmlMediaElement = document
            .get_element_by_id("video")
            .unwrap()
            .dyn_into()
            .unwrap(); */

            /* let callback = Closure::wrap(Box::new(on_abort) as Box<dyn Fn()>);
            video.set_onabort(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_canplay) as Box<dyn Fn()>);
            video.set_oncanplay(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_canplaythrough) as Box<dyn Fn()>);
            video.set_oncanplaythrough(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_durationchange) as Box<dyn Fn()>);
            video.set_ondurationchange(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_emptied) as Box<dyn Fn()>);
            video.set_onemptied(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_ended) as Box<dyn Fn()>);
            video.set_onended(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_ended) as Box<dyn Fn()>);
            video.set_onended(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_error) as Box<dyn Fn()>);
            video.set_onerror(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_loadeddata) as Box<dyn Fn()>);
            video.set_onloadeddata(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_loadedmetadata) as Box<dyn Fn()>);
            video.set_onloadedmetadata(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_loadstart) as Box<dyn Fn()>);
            video.set_onloadstart(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_pause) as Box<dyn Fn()>);
            video.set_onpause(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_play) as Box<dyn Fn()>);
            video.set_onplay(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_playing) as Box<dyn Fn()>);
            video.set_onplaying(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_progress) as Box<dyn Fn()>);
            video.set_onprogress(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_ratechange) as Box<dyn Fn()>);
            video.set_onratechange(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_seeked) as Box<dyn Fn()>);
            video.set_onseeked(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_seeking) as Box<dyn Fn()>);
            video.set_onseeking(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_stalled) as Box<dyn Fn()>);
            video.set_onstalled(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_suspend) as Box<dyn Fn()>);
            video.set_onsuspend(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_timeupdate) as Box<dyn Fn()>);
            video.set_ontimeupdate(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_volumechange) as Box<dyn Fn()>);
            video.set_onvolumechange(Some(callback.into_js_value().unchecked_ref()));

            let callback = Closure::wrap(Box::new(on_waiting) as Box<dyn Fn()>);
            video.set_onwaiting(Some(callback.into_js_value().unchecked_ref())); */

            //https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.HtmlVideoElement.html#method.set_src
            //video.set_src(&self.manager.url);

            //video.set_autoplay(false);
            //video.set_controls(true);
            //video.set_muted(true);

            //self.video = Some(video);
        }
    }
}

/* fn on_abort() {
    #[cfg(debug_assertions)]
    ConsoleService::info("abort");
} */

/* fn on_canplay() {
    #[cfg(debug_assertions)]
    ConsoleService::info("canplay");
} */

/* fn on_canplaythrough() {
    #[cfg(debug_assertions)]
    ConsoleService::info("canplaythrough");
} */

/* fn on_durationchange() {
    #[cfg(debug_assertions)]
    ConsoleService::info("durationchange");
} */

/* fn on_emptied() {
    #[cfg(debug_assertions)]
    ConsoleService::info("emptied");
} */

/* fn on_ended() {
    #[cfg(debug_assertions)]
    ConsoleService::info("ended");
} */

/* fn on_error() {
    #[cfg(debug_assertions)]
    ConsoleService::info("error");
} */

/* fn on_loadeddata() {
    #[cfg(debug_assertions)]
    ConsoleService::info("loadeddata");
} */

/* fn on_loadedmetadata() {
    #[cfg(debug_assertions)]
    ConsoleService::info("loadedmetadata");
} */

/* fn on_loadstart() {
    #[cfg(debug_assertions)]
    ConsoleService::info("loadstart");
} */

/* fn on_pause() {
    #[cfg(debug_assertions)]
    ConsoleService::info("pause");
} */

/* fn on_play() {
    #[cfg(debug_assertions)]
    ConsoleService::info("play");
} */

/* fn on_playing() {
    #[cfg(debug_assertions)]
    ConsoleService::info("playing");
} */

/* fn on_progress() {
    #[cfg(debug_assertions)]
    ConsoleService::info("progress");
} */

/* fn on_ratechange() {
    #[cfg(debug_assertions)]
    ConsoleService::info("ratechange");
} */

/* fn on_seeked() {
    #[cfg(debug_assertions)]
    ConsoleService::info("seeked");
} */

/* fn on_seeking() {
    #[cfg(debug_assertions)]
    ConsoleService::info("seeking");
} */

/* fn on_stalled() {
    #[cfg(debug_assertions)]
    ConsoleService::info("stalled");
} */

/* fn on_suspend() {
    #[cfg(debug_assertions)]
    ConsoleService::info("suspend");
} */

/* fn on_timeupdate() {
    #[cfg(debug_assertions)]
    ConsoleService::info("timeupdate");
} */

/* fn on_volumechange() {
    #[cfg(debug_assertions)]
    ConsoleService::info("volumechange");
} */

/* fn on_waiting() {
    #[cfg(debug_assertions)]
    ConsoleService::info("waiting");
} */
