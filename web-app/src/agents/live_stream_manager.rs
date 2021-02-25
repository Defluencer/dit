use std::collections::VecDeque;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use crate::utils::ema::ExponentialMovingAverage;
use crate::utils::ipfs::{cat_and_buffer, ipfs_dag_get_path_async};
use crate::utils::tracks::{Track, Tracks};

use futures_util::StreamExt;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use wasm_bindgen_futures::spawn_local;

use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, SourceBuffer, Url, Window};

use yew::services::ConsoleService;

use linked_data::video::SetupNode;

use cid::Cid;

use ipfs_api::IpfsClient;

//const CODEC_PATH: &str = "/setup/codec";
//const QUALITY_PATH: &str = "/setup/quality";

const BUFFER_LENGTH: f64 = 30.0;
const MEDIA_LENGTH: f64 = 4.0;

type Buffer = Arc<RwLock<VecDeque<Cid>>>;

//TODO add types common to live and vod then deduplicate fonctions.

#[derive(Clone)]
struct LiveStream {
    window: Window,
    video: Option<HtmlMediaElement>,
    media_source: MediaSource,

    tracks: Tracks,

    state: Arc<AtomicUsize>,
    level: Arc<AtomicUsize>,
    buffer: Buffer,

    handle: Arc<AtomicIsize>,

    ema: ExponentialMovingAverage,

    ipfs: IpfsClient,
}

pub struct LiveStreamManager {
    stream: LiveStream,

    topic: String,

    url: String,
}

impl LiveStreamManager {
    /// Ready Live Stream to link with video element.
    pub fn new(topic: String, streamer_peer_id: String) -> Self {
        let ipfs = IpfsClient::default();

        let buffer = Arc::new(RwLock::new(VecDeque::with_capacity(5)));

        ipfs_pubsub(&ipfs, &topic, buffer.clone(), streamer_peer_id);

        let window = web_sys::window().expect("Can't get window");

        let ema = ExponentialMovingAverage::new(&window);

        let media_source = MediaSource::new().expect("Can't create media source");

        let url = Url::create_object_url_with_source(&media_source)
            .expect("Can't create url from source");

        let stream = LiveStream {
            buffer,

            window,
            media_source,
            video: None,

            tracks: Arc::new(RwLock::new(Vec::with_capacity(4))),

            state: Arc::new(AtomicUsize::new(0)),
            level: Arc::new(AtomicUsize::new(0)),

            handle: Arc::new(AtomicIsize::new(0)),

            ema,

            ipfs,
        };

        Self { stream, topic, url }
    }

    /// Get video element and set source.
    pub fn link_video(&mut self) {
        let document = self.stream.window.document().expect("Can't get document");

        let video: HtmlMediaElement = document
            .get_element_by_id("video_player")
            .expect("No element with this Id")
            .dyn_into()
            .expect("Not Media Element");

        self.stream.video = Some(video.clone());

        video.set_src(&self.url);

        tick(self.stream.clone());
    }
}

impl Drop for LiveStreamManager {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping LiveStreamManager");

        //ipfs_unsubscribe(&self.topic);

        let handle = self.stream.handle.load(Ordering::Relaxed);

        if handle != 0 {
            self.stream.window.clear_interval_with_handle(handle as i32);
        }
    }
}

/// Update state machine.
fn tick(stream: LiveStream) {
    if stream.media_source.ready_state() != MediaSourceReadyState::Open {
        #[cfg(debug_assertions)]
        ConsoleService::info("Media Source Not Open");

        on_timeout(stream);

        return;
    }

    let current_state = stream.state.load(Ordering::Relaxed);

    match current_state {
        0 => spawn_local(add_source_buffer(stream)),
        1 => load_media_segment(stream),
        2 => spawn_local(switch_quality(stream)),
        3 => flush_back_buffer(stream),
        _ => {}
    }
}

/// Wait 1 second then check status again.
fn on_timeout(stream: LiveStream) {
    let window = stream.window.clone();
    let hanlde = stream.handle.clone();

    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Timeout");

        tick(stream.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);

    match window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.into_js_value().unchecked_ref(),
        1000,
    ) {
        Ok(handle) => hanlde.store(handle as isize, Ordering::Relaxed),
        Err(e) => ConsoleService::error(&format!("{:?}", e)),
    }
}

/// Get setup infos, create source buffer then load initialization segment.
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

    let setup_node: SetupNode =
        match ipfs_dag_get_path_async(stream.ipfs.clone(), &cid.to_string()).await {
            Ok(sn) => sn,
            Err(_) => return,
        };

    let mut vec = Vec::with_capacity(4);

    let first_codec = setup_node.codecs.first().expect("Can't get first codec");

    let source_buffer = match stream.media_source.add_source_buffer(first_codec) {
        Ok(sb) => sb,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    };

    #[cfg(debug_assertions)]
    ConsoleService::info("Listing Tracks");

    for (level, (codec, quality)) in setup_node
        .codecs
        .into_iter()
        .zip(setup_node.qualities.into_iter())
        .enumerate()
    {
        if !MediaSource::is_type_supported(&codec) {
            ConsoleService::error(&format!("MIME Type {:?} unsupported", &codec));
            return;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Level {} Quality {} Codec {}",
            level, quality, codec
        ));

        let track = Track {
            level,
            quality,
            codec,
            source_buffer: source_buffer.clone(),
        };

        vec.push(track);
    }

    on_source_buffer_update_end(stream.clone(), &source_buffer);

    let path = format!("{}/setup/initseg/{}", cid, 0);

    cat_and_buffer(stream.ipfs.clone(), path, source_buffer.clone()).await;

    match stream.tracks.write() {
        Ok(mut tracks) => *tracks = vec,
        Err(e) => {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }
    }

    //state load segment
    stream.state.store(1, Ordering::Relaxed);
}

/// Get CID from buffer then fetch data from ipfs
fn load_media_segment(stream: LiveStream) {
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

    let level = stream.level.load(Ordering::Relaxed);

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

    let path = format!("{}/quality/{}", cid, quality);

    let future = cat_and_buffer(stream.ipfs.clone(), path, source_buffer);

    stream.ema.start_timer();

    spawn_local(future);
}

/// Switch source buffer codec then load initialization segment.
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

    let level = stream.level.load(Ordering::Relaxed);

    let (quality, codec, source_buffer) = match stream.tracks.read() {
        Ok(tracks) => (
            tracks[level].quality.clone(),
            tracks[level].codec.clone(),
            tracks[level].source_buffer.clone(),
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

    cat_and_buffer(stream.ipfs.clone(), path, source_buffer).await;

    //state load segment
    stream.state.store(1, Ordering::Relaxed);
}

/// Flush last media segment.
fn flush_back_buffer(stream: LiveStream) {
    #[cfg(debug_assertions)]
    ConsoleService::info("Flushing Back Buffer");

    let level = stream.level.load(Ordering::Relaxed);

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

    if let Err(e) = source_buffer.remove(buff_start, buff_start + MEDIA_LENGTH) {
        ConsoleService::error(&format!("{:?}", e));
        return;
    }

    //state load segment
    stream.state.store(1, Ordering::Relaxed);
}

fn on_source_buffer_update_end(stream: LiveStream, source_buffer: &SourceBuffer) {
    let closure = move || {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Update End");

        let level = stream.level.load(Ordering::Relaxed);

        let source_buffer = match stream.tracks.read() {
            Ok(tracks) => tracks[level].source_buffer.clone(),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        if let Ok(time_ranges) = source_buffer.buffered() {
            let mut buff_start = 0.0;
            let mut buff_end = 0.0;

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

            if (buff_end - buff_start) > BUFFER_LENGTH {
                //state flush buffer
                stream.state.store(3, Ordering::Relaxed);
            }

            let current_time = stream.video.as_ref().unwrap().current_time();

            if current_time < buff_start {
                let new_time = buff_start + ((buff_end - buff_start) / 2.0);

                #[cfg(debug_assertions)]
                ConsoleService::info(&format!("Forward To {}s", new_time));

                stream.video.as_ref().unwrap().set_current_time(new_time);
            }
        }

        if let Some(moving_average) = stream.ema.recalculate_average() {
            match level {
                0 => {
                    if (moving_average + 500.0) < (MEDIA_LENGTH * 1000.0) {
                        stream.state.store(2, Ordering::Relaxed);
                        stream.level.store(level + 1, Ordering::Relaxed);
                    }
                }
                1 => {
                    if moving_average > (MEDIA_LENGTH * 1000.0) {
                        stream.state.store(2, Ordering::Relaxed);
                        stream.level.store(level - 1, Ordering::Relaxed);
                    } else if (moving_average + 500.0) < (MEDIA_LENGTH * 1000.0) {
                        stream.state.store(2, Ordering::Relaxed);
                        stream.level.store(level + 1, Ordering::Relaxed);
                    }
                }
                2 => {
                    if moving_average > (MEDIA_LENGTH * 1000.0) {
                        stream.state.store(2, Ordering::Relaxed);
                        stream.level.store(level - 1, Ordering::Relaxed);
                    } else if (moving_average + 500.0) < (MEDIA_LENGTH * 1000.0) {
                        stream.state.store(2, Ordering::Relaxed);
                        stream.level.store(level + 1, Ordering::Relaxed);
                    }
                }
                3 => {
                    if moving_average > (MEDIA_LENGTH * 1000.0) {
                        stream.state.store(2, Ordering::Relaxed);
                        stream.level.store(level - 1, Ordering::Relaxed);
                    }
                }
                _ => {}
            }
        }

        tick(stream.clone());
    };

    let callback = Closure::wrap(Box::new(closure) as Box<dyn Fn()>);
    source_buffer.set_onupdateend(Some(callback.into_js_value().unchecked_ref()));
}

/// Process Pubsub message.
async fn ipfs_pubsub(ipfs: &IpfsClient, topic: &str, buffer: Buffer, streamer_peer_id: String) {
    let mut stream = ipfs.pubsub_sub(topic, true);

    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                #[cfg(debug_assertions)]
                ConsoleService::info("PubSub Message");

                let from = match response.from {
                    Some(from) => from,
                    None => return,
                };

                #[cfg(debug_assertions)]
                ConsoleService::info(&format!("Sender => {}", from));

                if from != streamer_peer_id {
                    #[cfg(debug_assertions)]
                    ConsoleService::warn("Unauthorized Sender");
                    return;
                }

                let data = match response.data {
                    Some(data) => data,
                    None => return,
                };

                let cid = match decode_cid(data) {
                    Some(cid) => cid,
                    None => return,
                };

                //TODO get missing cid by checking previous value
                if let Ok(mut buffer) = buffer.write() {
                    buffer.push_back(cid);
                }
            }
            Err(error) => {
                eprintln!("{}", error);
                continue;
            }
        }
    }
}

/// Decode vector of byte into CID
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
