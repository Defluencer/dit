use std::collections::VecDeque;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use crate::bindings;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;

use serde_wasm_bindgen::from_value;

use js_sys::Uint8Array;
use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, SourceBuffer, Url, Window};

use yew::services::ConsoleService;

use cid::Cid;

const STREAMER_PEER_ID: &str = "12D3KooWAPZ3QZnZUJw3BgEX9F7XL383xFNiKQ5YKANiRC3NWvpo";
const CODEC_PATH: &str = "/setup/codec";
const QUALITY_PATH: &str = "/setup/quality";

type Buffer = Arc<RwLock<VecDeque<Cid>>>;

type Tracks = Arc<RwLock<Vec<Track>>>;

struct Track {
    //level: usize,
    quality: String,
    //codec: String,
    source_buffer: SourceBuffer,
}

pub fn load_live_stream(topic: String) {
    let buffer = Arc::new(RwLock::new(VecDeque::with_capacity(5)));

    ipfs_pubsub(&topic, buffer.clone());

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let media_source = MediaSource::new().unwrap();

    let url = Url::create_object_url_with_source(&media_source).unwrap();

    let video: HtmlMediaElement = document
        .get_element_by_id("video")
        .unwrap()
        .dyn_into()
        .unwrap();

    video.set_src(&url);

    let tracks = Arc::new(RwLock::new(Vec::with_capacity(4)));
    let current_level = Arc::new(AtomicUsize::new(0));

    let _id = match on_timer(media_source, buffer, window, tracks, current_level) {
        Ok(id) => id,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };
}

//TODO livecycle

fn on_timer(
    media_source: MediaSource,
    buffer: Buffer,
    window: Window,
    tracks: Tracks,
    current_level: Arc<AtomicUsize>,
) -> Result<i32, JsValue> {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("on timer");

        if media_source.ready_state() != MediaSourceReadyState::Open {
            #[cfg(debug_assertions)]
            ConsoleService::info("Media Source Not Open");

            return;
        }

        //Init
        if media_source.source_buffers().length() == 0 {
            let cid = match buffer.read() {
                Ok(buffer) => match buffer.front() {
                    Some(f) => f.to_string(),
                    None => return,
                },
                Err(e) => {
                    ConsoleService::error(&format!("{:?}", e));
                    return;
                }
            };

            let future = add_source_buffers(cid, media_source.clone(), tracks.clone());

            spawn_local(future);

            return;
        }

        let cid = match buffer.write() {
            Ok(mut buffer) => match buffer.pop_front() {
                Some(cid) => cid,
                None => return,
            },
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let tracks = tracks.read().expect("Lock Poisoned");
        let level = current_level.load(Ordering::Relaxed);

        let source_buffer = tracks[level].source_buffer.clone();
        let quality = &tracks[level].quality;

        let path = format!("{}/quality/{}", &cid.to_string(), quality);

        let future = cat_and_buffer(path, source_buffer);

        spawn_local(future);

        //TODO update adaptative bit rate
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        callback.into_js_value().unchecked_ref(),
        1000,
    )
}

async fn add_source_buffers(cid_string: String, media_source: MediaSource, tracks: Tracks) {
    let codecs = match bindings::ipfs_dag_get(&cid_string, CODEC_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let qualities = match bindings::ipfs_dag_get(&cid_string, QUALITY_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let codecs: Vec<String> = from_value(codecs).expect("Can't deserialize codecs");
    let qualities: Vec<String> = from_value(qualities).expect("Can't deserialize qualities");

    #[cfg(debug_assertions)]
    ConsoleService::info("Creating all source buffers");

    match tracks.write() {
        Ok(mut tracks) => {
            for (codec, quality) in codecs.into_iter().zip(qualities.into_iter()) {
                if !MediaSource::is_type_supported(&codec) {
                    ConsoleService::error(&format!("MIME Type {:?} unsupported", &codec));
                    continue;
                }

                let source_buffer = media_source
                    .add_source_buffer(&codec)
                    .expect("Can't add source buffer");

                let track = Track {
                    quality,
                    source_buffer,
                };

                tracks.push(track);
            }
        }
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }

    #[cfg(debug_assertions)]
    ConsoleService::info("Loading all initialization segments");

    match tracks.read() {
        Ok(tracks) => {
            for (level, track) in tracks.iter().enumerate() {
                let path = format!("{}/setup/initseg/{}", &cid_string, level);

                cat_and_buffer(path, track.source_buffer.clone()).await;
            }
        }
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }
}

fn ipfs_pubsub(topic: &str, buffer: Buffer) {
    let closure = move |from, data| {
        let cid = match decode_cid(from, data) {
            Some(cid) => cid,
            None => return,
        };

        //TODO get missing cid by checking previous value

        if let Ok(mut buffer) = buffer.write() {
            buffer.push_back(cid);
        }
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn(String, Vec<u8>)>);
    bindings::ipfs_subscribe(topic.into(), callback.into_js_value().unchecked_ref());
}

async fn cat_and_buffer(path: String, source_buffer: SourceBuffer) {
    let segment = match bindings::ipfs_cat(&path).await {
        Ok(vs) => vs,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let segment: &Uint8Array = segment.unchecked_ref();

    wait_for_buffer(source_buffer.clone()).await;

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(segment) {
        ConsoleService::warn(&format!("{:?}", e));
        return;
    }
}

async fn wait_for_buffer(source_buffer: SourceBuffer) {
    let closure = move || !source_buffer.updating();

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn() -> bool>);

    bindings::wait_until(callback.into_js_value().unchecked_ref()).await
}

fn decode_cid(from: String, data: Vec<u8>) -> Option<Cid> {
    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Sender => {}", from));

    if from != STREAMER_PEER_ID {
        #[cfg(debug_assertions)]
        ConsoleService::warn("Unauthorized Sender");

        return None;
    }

    let data_utf8 = match String::from_utf8(data) {
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

//Rebuild playlists by following the dag node link chain.
/* async fn rebuild_playlists(
    latest_dag_node: DagNode,
    playlists: &Arc<RwLock<Playlists>>,
    previous_cid: &Option<String>,
    ipfs: &IpfsClient,
) {
    let mut missing_nodes = Vec::with_capacity(HLS_LIST_SIZE);

    missing_nodes.push(latest_dag_node);

    while missing_nodes.last().unwrap().previous != *previous_cid {
        //Fill the vec with all the missing nodes.

        let dag_node_cid = match missing_nodes.last().unwrap().previous.as_ref() {
            Some(cid) => cid,
            None => {
                //Found first node of the stream, stop here.
                break;
            }
        };

        let dag_node = match get_dag_node(ipfs, dag_node_cid).await {
            Ok(data) => data,
            Err(error) => {
                eprintln!("IPFS dag get failed {}", error);
                return;
            }
        };

        missing_nodes.push(dag_node);

        if missing_nodes.len() >= HLS_LIST_SIZE {
            //Found more node than the list size, stop here.
            break;
        }
    }

    let mut playlists = playlists.write().await;

    for dag_node in missing_nodes.into_iter().rev() {
        #[cfg(debug_assertions)]
        println!("Missing {:#?}", &dag_node);

        update_playlists(dag_node, &mut playlists);
    }
} */
