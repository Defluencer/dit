use crate::bindings;

//use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use js_sys::Uint8Array;
use web_sys::{HtmlMediaElement, MediaSource, SourceBuffer, Url};

use yew::services::ConsoleService;

//use cid::Cid;

const STREAMER_PEER_ID: &str = "12D3KooWAPZ3QZnZUJw3BgEX9F7XL383xFNiKQ5YKANiRC3NWvpo";
const MIME_TYPE: &str = r#"video/mp4; codecs="avc1.42c01f, mp4a.40.2""#;

pub fn load_live_stream(topic: String) {
    if !MediaSource::is_type_supported(MIME_TYPE) {
        ConsoleService::warn(&format!("MIME Type {:?} unsupported", MIME_TYPE));
        return;
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let video: HtmlMediaElement = document
        .get_element_by_id("video")
        .unwrap()
        .dyn_into()
        .unwrap();

    let media_source = MediaSource::new().unwrap();

    let url = Url::create_object_url_with_source(&media_source).unwrap();
    video.set_src(&url);

    let initialized = Arc::new(AtomicBool::new(false));

    media_source_on_source_open(topic, initialized, media_source);
}

fn media_source_on_source_open(
    topic: String,
    initialized: Arc<AtomicBool>,
    media_source: MediaSource,
) {
    let media_source_clone = media_source.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("sourceopen");

        let source_buffer = match media_source.add_source_buffer(MIME_TYPE) {
            Ok(sb) => sb,
            Err(e) => {
                ConsoleService::warn(&format!("{:?}", e));
                return;
            }
        };

        let initialized = initialized.clone();

        ipfs_sub(&topic, initialized, source_buffer);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    media_source_clone.set_onsourceopen(Some(callback.into_js_value().unchecked_ref()));
}

fn ipfs_sub(topic: &str, initialized: Arc<AtomicBool>, source_buffer: SourceBuffer) {
    let closure = move |from, data| {
        let cid = match decode_cid(from, data) {
            Some(cid) => cid,
            None => return,
        };

        if !initialized.compare_and_swap(false, true, Ordering::SeqCst) {
            let path = &format!("{}/init/720p30", &cid);

            let future = cat_and_buffer(path, source_buffer.clone());

            spawn_local(future);
        }

        let path = &format!("{}/quality/720p30", &cid);

        let future = cat_and_buffer(path, source_buffer.clone());

        spawn_local(future);
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn(String, Vec<u8>)>);
    bindings::ipfs_subscribe(topic.into(), callback.into_js_value().unchecked_ref());
}

async fn cat_and_buffer(path: &str, source_buffer: SourceBuffer) {
    let segment = match bindings::ipfs_cat(path).await {
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

fn decode_cid(from: String, data: Vec<u8>) -> Option<String> {
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

    /* let video_cid = match Cid::try_from(data_utf8) {
        Ok(cid) => cid,
        Err(_) => {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Message Invalid CID");

            return None;
        }
    }; */

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!("Message => {}", data_utf8));

    Some(data_utf8)
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
