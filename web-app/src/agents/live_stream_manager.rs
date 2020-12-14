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

pub struct LiveStream {
    buffer: Buffer,

    tracks: Tracks,

    level: Arc<AtomicUsize>,

    window: Window,

    video: Option<HtmlMediaElement>,

    media_source: MediaSource,

    interval_handle: i32,
    callback: Closure<dyn Fn()>,

    topic: String,

    url: String,
}

impl LiveStream {
    pub fn new(topic: &str, streamer_peer_id: &str) -> Self {
        let buffer = Arc::new(RwLock::new(VecDeque::with_capacity(5)));

        ipfs_pubsub(topic, buffer.clone(), streamer_peer_id.into());

        let window = web_sys::window().expect("Can't get window");

        let media_source = MediaSource::new().expect("Can't create media source");

        let url = Url::create_object_url_with_source(&media_source)
            .expect("Can't create url from source");

        let tracks = Arc::new(RwLock::new(Vec::with_capacity(4)));
        let level = Arc::new(AtomicUsize::new(0));

        let (interval_handle, callback) = on_interval(
            media_source.clone(),
            buffer.clone(),
            window.clone(),
            tracks.clone(),
            level.clone(),
        )
        .expect("Can't start interval");

        Self {
            buffer,
            tracks,
            level,
            window,
            video: None,
            media_source,
            interval_handle,
            callback,
            topic: topic.into(),
            url,
        }
    }

    pub fn link_video(&mut self) {
        let document = self.window.document().expect("Can't get document");

        let video: HtmlMediaElement = document
            .get_element_by_id("video")
            .expect("No element with this Id")
            .unchecked_into();

        video.set_src(&self.url);

        self.video = Some(video);
    }
}

impl Drop for LiveStream {
    fn drop(&mut self) {
        bindings::ipfs_unsubscribe(self.topic.clone().into());

        self.window.clear_interval_with_handle(self.interval_handle);
    }
}

type IntervalResult = Result<(i32, Closure<dyn Fn()>), JsValue>;

fn on_interval(
    media_source: MediaSource,
    buffer: Buffer,
    window: Window,
    tracks: Tracks,
    current_level: Arc<AtomicUsize>,
) -> IntervalResult {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("on interval");

        if media_source.ready_state() != MediaSourceReadyState::Open {
            #[cfg(debug_assertions)]
            ConsoleService::info("Media Source Not Open");

            return;
        }

        if media_source.source_buffers().length() == 0 {
            //Initialize stream

            let cid = match buffer.read() {
                Ok(buffer) => match buffer.front() {
                    Some(f) => f.to_string(),
                    None => return, //nothing to update
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

        //Update stream

        let cid = match buffer.write() {
            Ok(mut buffer) => match buffer.pop_front() {
                Some(cid) => cid,
                None => return, //nothing to update
            },
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let level = current_level.load(Ordering::Relaxed);

        let (quality, source_buffer) = match tracks.read() {
            Ok(tracks) => (
                tracks[level].quality.clone(),
                tracks[level].source_buffer.clone(),
            ),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let path = format!("{}/quality/{}", &cid.to_string(), &quality);

        let future = cat_and_buffer(path, source_buffer);

        spawn_local(future);

        //TODO update adaptative bit rate
    };

    //Could use a loop with setTimeout()
    //that way the previous loop is garanteed to have finished before next call.

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);

    let handle = window.set_interval_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        1000,
    )?;

    Ok((handle, callback))
}

fn ipfs_pubsub(topic: &str, buffer: Buffer, streamer_peer_id: String) {
    let closure = move |from: String, data: Vec<u8>| {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        if from != streamer_peer_id {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Unauthorized Sender");
            return;
        }

        let cid = match decode_cid(data) {
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
                #[cfg(debug_assertions)]
                ConsoleService::info(&format!("Quality {} Codec {}", quality, codec));

                if !MediaSource::is_type_supported(&codec) {
                    ConsoleService::error(&format!("MIME Type {:?} unsupported", &codec));
                    continue;
                }

                let source_buffer = match media_source.add_source_buffer(&codec) {
                    Ok(sb) => sb,
                    Err(e) => {
                        ConsoleService::error(&format!("{:?}", e));
                        return;
                    }
                };

                #[cfg(debug_assertions)]
                ConsoleService::info(&format!(
                    "{} {} Buffer Mode {:#?}",
                    quality,
                    codec,
                    source_buffer.mode()
                ));

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

fn decode_cid(data: Vec<u8>) -> Option<Cid> {
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
