use crate::bindings;
use crate::playlists::Playlists;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

const TOPIC: &str = "livelikevideo";

#[derive(Clone)]
pub struct LiveStreamManager {
    playlists: Arc<RwLock<Playlists>>,

    initialized: Arc<AtomicBool>,

    loaded: Arc<AtomicBool>,
}

impl LiveStreamManager {
    pub fn new() -> Self {
        let playlists = Arc::new(RwLock::new(Playlists::new()));
        let playlists_clone = playlists.clone();

        let initialized = Arc::new(AtomicBool::new(false));
        let initialized_clone = initialized.clone();

        let loaded = Arc::new(AtomicBool::new(false));
        let loaded_clone = loaded.clone();

        let pubsub_closure = Closure::wrap(Box::new(move |from, data| {
            if let Ok(mut playlists) = playlists_clone.write() {
                playlists.pubsub_message(from, data);

                if !loaded_clone.compare_and_swap(false, true, Ordering::Relaxed)
                    && initialized_clone.load(Ordering::Relaxed)
                    && playlists.has_segments()
                {
                    bindings::start_load();
                }
            }
        }) as Box<dyn Fn(String, Vec<u8>)>);

        bindings::subscribe(TOPIC.into(), pubsub_closure.into_js_value().unchecked_ref());

        Self {
            playlists,
            initialized,
            loaded,
        }
    }

    pub fn init_hls(&self) {
        let playlists_clone = self.playlists.clone();

        let playlist_closure = Closure::wrap(Box::new(move |url| match playlists_clone.read() {
            Ok(playlists) => playlists.get_playlist(url),
            Err(_) => String::from("Lock Poisoned"),
        }) as Box<dyn Fn(String) -> String>);

        bindings::init_hls(playlist_closure.into_js_value().unchecked_ref());

        self.initialized.store(true, Ordering::Relaxed);
    }
}
