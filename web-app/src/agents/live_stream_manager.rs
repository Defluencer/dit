use crate::bindings;
use crate::playlists::Playlists;

use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use yew::services::ConsoleService;

use cid::Cid;

const TOPIC: &str = "livelikevideo";
const STREAMER_PEER_ID: &str = "12D3KooWAPZ3QZnZUJw3BgEX9F7XL383xFNiKQ5YKANiRC3NWvpo";

/// Initialized HLS, subscribe to video updates. Callbacks ownership is then transfered to JS GC.
pub fn load_live_stream() {
    let live_playlists = Arc::new(RwLock::new(Playlists::new()));

    let playlists = live_playlists.clone();

    let playlist_closure = Closure::wrap(Box::new(move |path| match playlists.read() {
        Ok(playlists) => playlists.get_playlist(path),
        Err(_) => String::from("RwLock Poisoned"),
    }) as Box<dyn Fn(String) -> String>);

    bindings::register_playlist_callback(playlist_closure.into_js_value().unchecked_ref());

    bindings::init_hls();

    bindings::hls_load_master_playlist();

    let loaded = Arc::new(AtomicBool::new(false));

    let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
        let cid = match pubsub_video(from, data) {
            Some(cid) => cid,
            None => return,
        };

        match live_playlists.write() {
            Ok(mut playlist) => playlist.update_live_playlists(&cid),
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::error("RwLock Poisoned");

                return;
            }
        }

        if !loaded.compare_and_swap(false, true, Ordering::Relaxed) {
            bindings::hls_start_load();
        }
    }) as Box<dyn Fn(String, Vec<u8>)>);

    bindings::subscribe(TOPIC.into(), pubsub_closure.into_js_value().unchecked_ref());
}

// TODO make sure registering callbacks is not every time the page is loaded

/// Destroy HLS, unsubscribe from video updates then JS GC free callbacks
pub fn unload_live_stream() {
    bindings::hls_destroy();

    bindings::unregister_playlist_callback();

    bindings::unsubscribe(TOPIC.into());
}

fn pubsub_video(from: String, data: Vec<u8>) -> Option<Cid> {
    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Sender => {}", from));

    if from != STREAMER_PEER_ID {
        #[cfg(debug_assertions)]
        ConsoleService::warn("Unauthorized Sender");

        return None;
    }

    let data_utf8 = match std::str::from_utf8(&data) {
        Ok(string) => string,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Message Invalid UTF-8");

            return None;
        }
    };

    let video_cid = match Cid::try_from(data_utf8) {
        Ok(cid) => cid,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Message Invalid CID");

            return None;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Message => {}", video_cid));

    Some(video_cid)
}
