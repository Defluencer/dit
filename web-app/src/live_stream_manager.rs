use crate::bindings;
use crate::playlists::Playlists;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

const TOPIC: &str = "livelikevideo";

#[derive(Clone)]
pub struct LiveStreamManager {
    live_playlists: Arc<RwLock<Playlists>>,

    initialized: Arc<AtomicBool>,

    loaded: Arc<AtomicBool>,
}

impl LiveStreamManager {
    pub fn new() -> Self {
        let live_playlists = Arc::new(RwLock::new(Playlists::new()));

        let initialized = Arc::new(AtomicBool::new(false));
        let loaded = Arc::new(AtomicBool::new(false));

        Self {
            live_playlists,
            initialized,
            loaded,
        }
    }

    pub fn playlists_updating(&self) {
        let playlists = self.live_playlists.clone();

        let loaded = self.loaded.clone();
        let initialized = self.initialized.clone();

        let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
            if let Ok(mut playlists) = playlists.write() {
                playlists.pubsub_message(from, data)
            }

            if initialized.load(Ordering::Relaxed)
                && !loaded.compare_and_swap(false, true, Ordering::Relaxed)
            {
                bindings::start_load();
            }
        }) as Box<dyn Fn(String, Vec<u8>)>);

        bindings::subscribe(TOPIC.into(), pubsub_closure.into_js_value().unchecked_ref());
    }

    pub fn register_callback(&self) {
        let playlists = self.live_playlists.clone();

        let playlist_closure = Closure::wrap(Box::new(move |path| match playlists.read() {
            Ok(playlists) => playlists.get_playlist(path),
            Err(_) => String::from("Lock Poisoned"),
        }) as Box<dyn Fn(String) -> String>);

        bindings::playlist_callback(playlist_closure.into_js_value().unchecked_ref());

        bindings::init_hls();

        bindings::load_source();

        self.initialized.store(true, Ordering::Relaxed);
    }
}
