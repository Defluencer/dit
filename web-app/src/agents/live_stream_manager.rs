use std::collections::VecDeque;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicI32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use crate::bindings;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use serde_wasm_bindgen::from_value;

use js_sys::Uint8Array;
use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, SourceBuffer, Url, Window};

use yew::services::ConsoleService;

use cid::Cid;

const CODEC_PATH: &str = "/setup/codec";
const QUALITY_PATH: &str = "/setup/quality";

const BUFFER_LENGTH: f64 = 30.0;
const MEDIA_LENGTH: f64 = 4.0;

//TODO flush when buffer is too big
//TODO update adaptative bit rate

type Buffer = Arc<RwLock<VecDeque<Cid>>>;

type Tracks = Arc<RwLock<Vec<Track>>>;

struct Track {
    level: usize,
    quality: String,
    codec: String,
    source_buffer: SourceBuffer,
}

#[derive(Clone)]
struct LiveStream {
    window: Window,

    video: Option<HtmlMediaElement>,

    media_source: MediaSource,

    tracks: Tracks,

    state: Arc<AtomicU8>,

    level: Arc<AtomicUsize>,

    buffer: Buffer,

    handle: Arc<AtomicI32>,
}

pub struct LiveStreamManager {
    stream: LiveStream,

    topic: String,

    url: String,
}

impl LiveStreamManager {
    pub fn new(topic: &str, streamer_peer_id: &str) -> Self {
        let buffer = Arc::new(RwLock::new(VecDeque::with_capacity(5)));

        ipfs_pubsub(topic, buffer.clone(), streamer_peer_id.into());

        let window = web_sys::window().expect("Can't get window");

        let media_source = MediaSource::new().expect("Can't create media source");

        let url = Url::create_object_url_with_source(&media_source)
            .expect("Can't create url from source");

        let tracks = Arc::new(RwLock::new(Vec::with_capacity(4)));
        let state = Arc::new(AtomicU8::new(0));
        let level = Arc::new(AtomicUsize::new(0));
        let handle = Arc::new(AtomicI32::new(0));

        let stream = LiveStream {
            buffer,
            tracks,
            state,
            level,
            window,
            video: None,
            media_source,
            handle,
        };

        Self {
            stream,
            topic: topic.into(),
            url,
        }
    }

    pub fn link_video(&mut self) {
        let document = self.stream.window.document().expect("Can't get document");

        let video: HtmlMediaElement = document
            .get_element_by_id("video")
            .expect("No element with this Id")
            .unchecked_into();

        video.set_src(&self.url);

        self.stream.video = Some(video);

        self.test_level_switch();

        tick(self.stream.clone());
    }

    fn test_level_switch(&mut self) {
        let state = self.stream.state.clone();
        let level = self.stream.level.clone();

        let closure = move || {
            #[cfg(debug_assertions)]
            ConsoleService::info("TEST Level Switch");

            level.store(3, Ordering::SeqCst);
            state.store(2, Ordering::SeqCst);
        };

        let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);

        if let Err(e) = self
            .stream
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.into_js_value().unchecked_ref(),
                15000,
            )
        {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }
}

impl Drop for LiveStreamManager {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping LiveStreamManager");

        bindings::ipfs_unsubscribe(self.topic.clone().into());

        let handle = self.stream.handle.load(Ordering::SeqCst);

        if handle != 0 {
            self.stream.window.clear_interval_with_handle(handle);
        }
    }
}

fn on_source_buffer_update_end(stream: LiveStream, source_buffer: SourceBuffer) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Update End");

        tick(stream.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
}

fn on_timeout(stream: LiveStream) {
    let window_clone = stream.window.clone();
    let hanlde_clone = stream.handle.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Timeout");

        tick(stream.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);

    match window_clone.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.into_js_value().unchecked_ref(),
        1000,
    ) {
        Ok(handle) => hanlde_clone.store(handle, Ordering::SeqCst),
        Err(e) => ConsoleService::error(&format!("{:?}", e)),
    }
}

fn tick(stream: LiveStream) {
    if stream.media_source.ready_state() != MediaSourceReadyState::Open {
        #[cfg(debug_assertions)]
        ConsoleService::info("Media Source Not Open");

        on_timeout(stream);

        return;
    }

    let current_state = stream.state.load(Ordering::SeqCst);

    match current_state {
        //0 => create source buffer and load init seg
        0 => spawn_local(add_source_buffer(stream)),
        //1 = load media segment
        1 => load_segment(stream),
        //2 = switch quality
        2 => spawn_local(switch_quality(stream)),
        //3 = flush buffer
        3 => flush_back_buffer(stream),
        _ => {}
    }
}

async fn add_source_buffer(stream: LiveStream) {
    let cid = match stream.buffer.read() {
        Ok(buf) => match buf.front() {
            Some(cid) => cid.to_string(),
            None => {
                //No Cid available yet. Try again later!
                on_timeout(stream.clone());
                return;
            }
        },
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info("Adding Source Buffer");

    let codecs = match bindings::ipfs_dag_get(&cid, CODEC_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let qualities = match bindings::ipfs_dag_get(&cid, QUALITY_PATH).await {
        Ok(result) => result,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let codecs: Vec<String> = from_value(codecs).expect("Can't deserialize codecs");
    let qualities: Vec<String> = from_value(qualities).expect("Can't deserialize qualities");

    let init_codec = codecs.first().expect("No Codecs");

    if !MediaSource::is_type_supported(init_codec) {
        ConsoleService::error(&format!("MIME Type {:?} unsupported", init_codec));
        return;
    }

    let source_buffer = match stream.media_source.add_source_buffer(init_codec) {
        Ok(sb) => sb,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    on_source_buffer_update_end(stream.clone(), source_buffer.clone());

    let mut vec = Vec::with_capacity(4);

    #[cfg(debug_assertions)]
    ConsoleService::info("Listing Tracks");

    for (level, (codec, quality)) in codecs.into_iter().zip(qualities.into_iter()).enumerate() {
        if !MediaSource::is_type_supported(&codec) {
            ConsoleService::error(&format!("MIME Type {:?} unsupported", &codec));
            return;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Level {} Quality {} Codec {} Buffer Mode {:#?}",
            level,
            quality,
            codec,
            source_buffer.mode()
        ));

        let track = Track {
            level,
            quality,
            codec,
            source_buffer: source_buffer.clone(),
        };

        vec.push(track);
    }

    let track = &vec[0];

    let path = format!("{}/setup/initseg/{}", &cid, track.level);

    cat_and_buffer(path, source_buffer.clone()).await;

    match stream.tracks.write() {
        Ok(mut tracks) => *tracks = vec,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }

    //state load segment
    stream.state.store(1, Ordering::SeqCst);
}

fn load_segment(stream: LiveStream) {
    let cid = match stream.buffer.write() {
        Ok(mut buf) => match buf.pop_front() {
            Some(cid) => cid.to_string(),
            None => {
                //No Cid available yet. Try again later!
                on_timeout(stream.clone());
                return;
            }
        },
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info("Loading Media Segment");

    let level = stream.level.load(Ordering::SeqCst);

    let (quality, source_buffer) = match stream.tracks.read() {
        Ok(tracks) => (
            tracks[level].quality.clone(),
            tracks[level].source_buffer.clone(),
        ),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let mut buff_start = 0.0;
    let mut buff_end = 0.0;

    if let Ok(time_ranges) = source_buffer.buffered() {
        let count = time_ranges.length();

        for i in 0..count {
            if let Ok(start) = time_ranges.start(i) {
                buff_start = start;
            }

            if let Ok(end) = time_ranges.end(i) {
                buff_end = end;
            }

            #[cfg(debug_assertions)]
            ConsoleService::info(&format!(
                "Time Range {} buffers {}s to {}s",
                i, buff_start, buff_end
            ));
        }

        if (buff_end + MEDIA_LENGTH - buff_start) > BUFFER_LENGTH {
            //state flush buffer
            stream.state.store(3, Ordering::SeqCst);
        }

        let current_time = stream.video.as_ref().unwrap().current_time();

        if current_time < buff_start {
            //middle of buffer
            let new_time = buff_start + ((buff_end - buff_start) / 2.0);

            //TODO try new_time = buff-end - medai length

            #[cfg(debug_assertions)]
            ConsoleService::info(&format!("Forward To {}s", new_time));

            stream.video.unwrap().set_current_time(new_time);
        }
    }

    let path = format!("{}/quality/{}", cid, quality);

    let future = cat_and_buffer(path, source_buffer);

    spawn_local(future);
}

async fn switch_quality(stream: LiveStream) {
    let cid = match stream.buffer.read() {
        Ok(buf) => match buf.front() {
            Some(cid) => cid.to_string(),
            None => {
                //No Cid available yet. Try again later!
                on_timeout(stream.clone());
                return;
            }
        },
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info("Switching Quality");

    let level = stream.level.load(Ordering::SeqCst);

    let (quality, source_buffer, codec) = match stream.tracks.read() {
        Ok(tracks) => (
            tracks[level].quality.clone(),
            tracks[level].source_buffer.clone(),
            tracks[level].codec.clone(),
        ),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    if let Err(e) = source_buffer.change_type(&codec) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }

    #[cfg(debug_assertions)]
    ConsoleService::info(&format!(
        "Level {} Quality {} Codec {} Buffer Mode {:#?}",
        level,
        quality,
        codec,
        source_buffer.mode()
    ));

    let path = format!("{}/setup/initseg/{}", cid, level);

    cat_and_buffer(path, source_buffer.clone()).await;

    //state load segment
    stream.state.store(1, Ordering::SeqCst);
}

fn flush_back_buffer(stream: LiveStream) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Flushing Back Buffer");

    let level = stream.level.load(Ordering::SeqCst);

    let source_buffer = match stream.tracks.read() {
        Ok(tracks) => tracks[level].source_buffer.clone(),
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    let buff_start = source_buffer
        .buffered()
        .expect("No Buffer")
        .start(0)
        .expect("No TimeRange");

    let current_time = stream.video.unwrap().current_time();

    let flush_start = buff_start;

    let flush_end = current_time - MEDIA_LENGTH;

    if let Err(e) = source_buffer.remove(flush_start, flush_end) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }

    //state load segment
    stream.state.store(1, Ordering::SeqCst);
}

fn ipfs_pubsub(topic: &str, buffer: Buffer, streamer_peer_id: String) {
    let closure = move |from: String, data: Vec<u8>| {
        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message");

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

async fn cat_and_buffer(path: String, source_buffer: SourceBuffer) {
    let segment = match bindings::ipfs_cat(&path).await {
        Ok(vs) => vs,
        Err(e) => {
            ConsoleService::warn(&format!("{:?}", e));
            return;
        }
    };

    let segment: &Uint8Array = segment.unchecked_ref();

    //wait_for_buffer(source_buffer.clone()).await;

    if let Err(e) = source_buffer.append_buffer_with_array_buffer_view(segment) {
        ConsoleService::warn(&format!("{:?}", e));
        return;
    }
}

async fn _wait_for_buffer(source_buffer: SourceBuffer) {
    let closure = move || !source_buffer.updating();

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn() -> bool>);

    bindings::wait_until(callback.into_js_value().unchecked_ref()).await
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
