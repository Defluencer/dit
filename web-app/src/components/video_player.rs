use std::collections::VecDeque;
use std::rc::Rc;
use std::str;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::utils::ema::ExponentialMovingAverage;
use crate::utils::ipfs::{IpfsService, PubsubSubResponse};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, SourceBuffer, Url, Window};

use yew::prelude::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::video::{SetupNode, Track, VideoMetadata};

use cid::Cid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const FORWARD_BUFFER_LENGTH: f64 = 16.0;
const BACK_BUFFER_LENGTH: f64 = 8.0;

const SETUP_PATH: &str = "/time/hour/0/minute/0/second/0/video/setup";

enum MachineState {
    Load,
    Switch,
    Flush,
    Timeout,
    AdaptativeBitrate,
    Status,
}

struct MediaBuffers {
    audio: SourceBuffer,
    video: SourceBuffer,

    tracks: Vec<Track>,
}

struct LiveStream {
    streamer_peer_id: String,

    buffer: VecDeque<Cid>,

    drop_sig: Rc<AtomicBool>,
}

pub struct VideoPlayer {
    link: ComponentLink<Self>,

    ipfs: IpfsService,
    metadata: Option<VideoMetadata>,
    live_stream: Option<LiveStream>,

    window: Window,
    media_element: Option<HtmlMediaElement>,
    media_source: MediaSource,
    media_buffers: Option<MediaBuffers>,
    object_url: String,
    poster_link: String,

    /// Level >= 1 since 0 is audio
    level: usize,
    state: MachineState,
    ema: ExponentialMovingAverage,

    source_open_closure: Option<Closure<dyn Fn()>>,
    seeking_closure: Option<Closure<dyn Fn()>>,
    update_end_closure: Option<Closure<dyn Fn()>>,
    timeout_closure: Option<Closure<dyn Fn()>>,
    handle: i32,
}

pub enum Msg {
    SourceOpen,
    Seeking,
    UpdateEnd,
    Timeout,
    SetupNode(Result<SetupNode>),
    Append(Result<(Vec<u8>, Vec<u8>)>),
    AppendVideo(Result<Vec<u8>>),
    PubSub(Result<PubsubSubResponse>),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,
    pub metadata: Option<VideoMetadata>,
    pub topic: Option<String>,
    pub streamer_peer_id: Option<String>,
}

impl Component for VideoPlayer {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props {
            ipfs,
            metadata,
            topic,
            streamer_peer_id,
        } = props;

        let window = web_sys::window().expect("Can't get window");

        let ema = ExponentialMovingAverage::new(&window);

        let media_source = MediaSource::new().expect("Can't create media source");

        let object_url = Url::create_object_url_with_source(&media_source)
            .expect("Can't create url from source");

        let mut poster_link = String::from("ipfs://");

        if let Some(md) = metadata.as_ref() {
            poster_link.push_str(&md.image.link.to_string());
        } else {
            //TODO default thumbnail image
            poster_link.push_str("bafkreicovb5qdvrine4vidt77xahhvovahmekvsojbiqewp7ih7pzvnn7i");
        }

        let cb = link.callback(|_| Msg::SourceOpen);
        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
        media_source.set_onsourceopen(Some(closure.as_ref().unchecked_ref()));
        let source_open_closure = Some(closure);

        let live_stream = match topic {
            Some(topic) => {
                let client = ipfs.clone();
                let cb = link.callback(Msg::PubSub);
                let drop_sig = Rc::from(AtomicBool::new(false));
                let sig = drop_sig.clone();

                spawn_local(async move { client.pubsub_sub(topic, cb, sig).await });

                Some(LiveStream {
                    streamer_peer_id: streamer_peer_id.unwrap(),
                    buffer: VecDeque::with_capacity(5),
                    drop_sig,
                })
            }
            None => None,
        };

        Self {
            link,

            ipfs,
            metadata,
            live_stream,

            window,
            media_element: None,
            media_source,
            media_buffers: None,
            object_url,
            poster_link,

            level: 1, // start at 1 since 0 is audio
            state: MachineState::Timeout,
            ema,

            source_open_closure,
            seeking_closure: None,
            update_end_closure: None,
            timeout_closure: None,
            handle: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SourceOpen => self.on_source_open(),
            Msg::Seeking => self.on_seeking(),
            Msg::UpdateEnd => self.on_update_end(),
            Msg::Timeout => self.on_timeout(),
            Msg::SetupNode(result) => self.add_source_buffer(result),
            Msg::Append(result) => self.append_buffers(result),
            Msg::AppendVideo(result) => self.append_video_buffer(result),
            Msg::PubSub(result) => self.on_pubsub_update(result),
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <video class="video_player" id="video_player" autoplay=true controls=true poster=self.poster_link />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let document = self.window.document().expect("Can't get document");

            let media_element: HtmlMediaElement = document
                .get_element_by_id("video_player")
                .expect("No element with this Id")
                .dyn_into()
                .expect("Not Media Element");

            media_element.set_src(&self.object_url);

            self.seeking_closure = match self.metadata.as_ref() {
                Some(_) => {
                    let cb = self.link.callback(|_| Msg::Seeking);
                    let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
                    media_element.set_onseeking(Some(closure.as_ref().unchecked_ref()));

                    Some(closure)
                }
                None => None,
            };

            self.media_element = Some(media_element);
        }
    }

    fn destroy(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Dropping Video Player");

        if let Some(live) = self.live_stream.as_ref() {
            live.drop_sig.store(true, Ordering::Relaxed);
        }

        if self.handle != 0 {
            self.window.clear_timeout_with_handle(self.handle);
        }
    }
}

impl VideoPlayer {
    /// Callback when MediaSource is linked to video element.
    fn on_source_open(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Source Open");

        self.media_source.set_onsourceopen(None);
        self.source_open_closure = None;

        if let Some(metadata) = self.metadata.as_ref() {
            self.media_source.set_duration(metadata.duration);

            let cb = self.link.callback_once(Msg::SetupNode);
            let client = self.ipfs.clone();
            let cid = metadata.video.link;

            spawn_local(async move { cb.emit(client.dag_get(cid, Some(SETUP_PATH)).await) });
        }
    }

    /// Callback when GossipSub receive an update.
    fn on_pubsub_update(&mut self, result: Result<PubsubSubResponse>) {
        let res = match result {
            Ok(res) => res,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message Received");

        let PubsubSubResponse { from, data } = res;

        let live = self.live_stream.as_mut().unwrap();

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        if from != live.streamer_peer_id {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Unauthorized Sender");
            return;
        }

        let data = match str::from_utf8(&data) {
            Ok(data) => data,
            Err(e) => {
                #[cfg(debug_assertions)]
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Message => {}", data));

        let cid = match Cid::from_str(data) {
            Ok(cid) => cid,
            Err(e) => {
                #[cfg(debug_assertions)]
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        live.buffer.push_back(cid);

        if self.media_buffers.is_none() {
            let cb = self.link.callback_once(Msg::SetupNode);
            let client = self.ipfs.clone();

            spawn_local(async move { cb.emit(client.dag_get(cid, Some("/setup/")).await) });
        }
    }

    /// Callback when source buffer is done updating.
    fn on_update_end(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Update End");

        self.tick()
    }

    /// Callback when video element has seeked.
    fn on_seeking(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Seeking");

        self.state = MachineState::Flush;
    }

    /// Callback when 1 second has passed.
    fn on_timeout(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Timeout");

        self.timeout_closure = None;
        self.handle = 0;

        self.tick()
    }

    /// Update state machine.
    fn tick(&mut self) {
        match self.state {
            MachineState::Load => self.load_segment(),
            MachineState::Switch => self.switch_quality(),
            MachineState::Flush => self.flush_buffer(),
            MachineState::Timeout => self.set_timeout(),
            MachineState::Status => self.check_status(),
            MachineState::AdaptativeBitrate => self.check_abr(),
        }
    }

    /// Set 1 second timeout.
    fn set_timeout(&mut self) {
        if self.timeout_closure.is_some() {
            return;
        }

        let cb = self.link.callback_once(|_| Msg::Timeout);

        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);

        match self
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            ) {
            Ok(handle) => self.handle = handle,
            Err(e) => ConsoleService::error(&format!("{:?}", e)),
        }

        self.timeout_closure = Some(closure);
    }

    /// Create source buffer then load initialization segment.
    fn add_source_buffer(&mut self, setup_node: Result<SetupNode>) {
        let setup_node = match setup_node {
            Ok(n) => n,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Adding Source Buffer");

        if self.media_source.ready_state() != MediaSourceReadyState::Open {
            #[cfg(debug_assertions)]
            ConsoleService::info("Media Source Not Open");
            return;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Setup Node \n {}",
            &serde_json::to_string_pretty(&setup_node).expect("Can't print")
        ));

        #[cfg(debug_assertions)]
        ConsoleService::info("Listing Tracks");

        let mut audio_buffer = None;
        let mut video_buffer = None;

        for (level, track) in setup_node.tracks.iter().enumerate() {
            if !MediaSource::is_type_supported(&track.codec) {
                ConsoleService::error(&format!("MIME Type {:?} unsupported", &track.codec));
                continue;
            }

            #[cfg(debug_assertions)]
            ConsoleService::info(&format!(
                "Level {} Name {} Codec {} Bandwidth {}",
                level, track.name, track.codec, track.bandwidth
            ));

            if video_buffer.is_some() {
                continue;
            }

            if track.name == "audio" && audio_buffer.is_some() {
                continue;
            }

            let source_buffer = match self.media_source.add_source_buffer(&track.codec) {
                Ok(sb) => sb,
                Err(e) => {
                    ConsoleService::error(&format!("{:?}", e));
                    return;
                }
            };

            if track.name == "audio" {
                audio_buffer = Some(source_buffer);
            } else {
                video_buffer = Some(source_buffer);
            }
        }

        let media_buffer = MediaBuffers {
            audio: audio_buffer.unwrap(),
            video: video_buffer.unwrap(),
            tracks: setup_node.tracks,
        };

        let cb = self.link.callback(|_| Msg::UpdateEnd);
        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
        media_buffer
            .video
            .set_onupdateend(Some(closure.as_ref().unchecked_ref()));

        self.update_end_closure = Some(closure);

        let audio_path = media_buffer.tracks[0]
            .initialization_segment
            .link
            .to_string();
        let video_path = media_buffer.tracks[1]
            .initialization_segment
            .link
            .to_string();

        self.media_buffers = Some(media_buffer);
        self.state = MachineState::Load;

        let cb = self.link.callback_once(Msg::Append);
        let client = self.ipfs.clone();

        spawn_local(async move { cb.emit(client.double_path_cat(audio_path, video_path).await) });
    }

    /// Load either live or VOD segment.
    fn load_segment(&mut self) {
        if self.metadata.is_some() {
            return self.load_vod_segment();
        }

        self.load_live_segment()
    }

    /// Try get cid from live buffer then fetch video data from ipfs.
    fn load_live_segment(&mut self) {
        let live = self.live_stream.as_mut().unwrap();

        let cid = match live.buffer.pop_front() {
            Some(cid) => cid,
            None => return self.set_timeout(),
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Loading Live Media Segments");

        let track_name = &self.media_buffers.as_ref().unwrap().tracks[self.level].name;

        let audio_path = format!("{}/track/audio", cid.to_string());
        let video_path = format!("{}/track/{}", cid.to_string(), track_name);

        self.state = MachineState::AdaptativeBitrate;
        self.ema.start_timer();

        let cb = self.link.callback_once(Msg::Append);
        let client = self.ipfs.clone();

        spawn_local(async move { cb.emit(client.double_path_cat(audio_path, video_path).await) });
    }

    /// Get CID from timecode then fetch video data from ipfs.
    fn load_vod_segment(&mut self) {
        let metadata = self.metadata.as_ref().unwrap();
        let buffers = self.media_buffers.as_ref().unwrap();

        let track_name = &buffers.tracks[self.level].name;

        let time_ranges = match buffers.video.buffered() {
            Ok(tm) => tm,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::info("Buffer empty");
                return;
            }
        };

        let mut buff_end = 0.0;

        let count = time_ranges.length();

        if count > 0 {
            if let Ok(end) = time_ranges.end(count - 1) {
                buff_end = end;
            }
        }

        //if buffer is empty load at current time
        if buff_end <= 0.0 {
            let current_time = match self.media_element.as_ref() {
                Some(media_element) => media_element.current_time(),
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::info("No Media Element");
                    return;
                }
            };

            if current_time > 1.0 {
                buff_end = current_time - 1.0;
            }
        }

        let (hours, minutes, seconds) = seconds_to_timecode(buff_end);

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Loading Media Segments at timecode {}:{}:{}",
            hours, minutes, seconds
        ));

        let audio_path = format!(
            "{}/time/hour/{}/minute/{}/second/{}/video/track/audio",
            metadata.video.link.to_string(),
            hours,
            minutes,
            seconds,
        );

        let video_path = format!(
            "{}/time/hour/{}/minute/{}/second/{}/video/track/{}",
            metadata.video.link.to_string(),
            hours,
            minutes,
            seconds,
            track_name,
        );

        self.state = MachineState::AdaptativeBitrate;
        self.ema.start_timer();

        let cb = self.link.callback_once(Msg::Append);
        let client = self.ipfs.clone();

        spawn_local(async move { cb.emit(client.double_path_cat(audio_path, video_path).await) });
    }

    /// Recalculate download speed then set quality level.
    fn check_abr(&mut self) {
        let buffers = self.media_buffers.as_ref().unwrap();

        let bandwidth = buffers.tracks[self.level].bandwidth as f64;

        let avg_bitrate = match self.ema.recalculate_average_speed(bandwidth) {
            Some(at) => at,
            None => {
                self.state = MachineState::Status;
                return self.tick();
            }
        };

        let mut next_level = 1; // start at 1 since 0 is audio
        while let Some(next_bitrate) = buffers.tracks.get(next_level + 1).map(|t| t.bandwidth) {
            if avg_bitrate <= next_bitrate as f64 {
                break;
            }

            next_level += 1;
        }

        if next_level == self.level {
            self.state = MachineState::Status;
            return self.tick();
        }

        self.level = next_level;
        self.state = MachineState::Switch;
        self.tick()
    }

    /// Check buffers and current time then trigger new action.
    fn check_status(&mut self) {
        let buffers = self.media_buffers.as_ref().unwrap();

        let time_ranges = match buffers.video.buffered() {
            Ok(tm) => tm,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::info("Buffer empty");
                return self.set_timeout();
            }
        };

        let count = time_ranges.length();

        let mut buff_start = 0.0;
        let mut buff_end = 0.0;

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

        let current_time = match self.media_element.as_ref() {
            Some(media_element) => media_element.current_time(),
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::info("No Media Element");
                return self.set_timeout();
            }
        };

        if current_time < buff_start {
            let new_time = buff_start + ((buff_end - buff_start) / 2.0);

            #[cfg(debug_assertions)]
            ConsoleService::info(&format!("Forward To {}s", new_time));

            self.media_element
                .as_ref()
                .unwrap()
                .set_current_time(new_time);
        }

        if current_time > buff_start + BACK_BUFFER_LENGTH {
            #[cfg(debug_assertions)]
            ConsoleService::info("Back Buffer Full");
            return self.flush_buffer();
        }

        if self.metadata.is_some() && buff_end >= self.metadata.as_ref().unwrap().duration {
            #[cfg(debug_assertions)]
            ConsoleService::info("End Of Video");
            return;
        }

        if self.metadata.is_some() && current_time + FORWARD_BUFFER_LENGTH < buff_end {
            #[cfg(debug_assertions)]
            ConsoleService::info("Forward Buffer Full");
            return self.set_timeout();
        }

        self.load_segment()
    }

    /// Flush everything or just back buffer.
    fn flush_buffer(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Flushing Buffer");

        let buffers = self.media_buffers.as_ref().unwrap();

        let time_ranges = match buffers.video.buffered() {
            Ok(tm) => tm,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::info("Buffer empty");
                return;
            }
        };

        let count = time_ranges.length();

        let mut buff_start = 0.0;
        let mut buff_end = 0.0;

        if let Ok(start) = time_ranges.start(0) {
            buff_start = start;
        }

        if let Ok(end) = time_ranges.end(count - 1) {
            buff_end = end;
        }

        let current_time = match self.media_element.as_ref() {
            Some(media_element) => media_element.current_time(),
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::info("No Media Element");
                return;
            }
        };

        let back_buffer_start = current_time - BACK_BUFFER_LENGTH;

        //full flush except if back buffer flush is possible
        if buff_start < back_buffer_start {
            buff_end = back_buffer_start
        }

        if let Err(e) = buffers.audio.remove(buff_start, buff_end) {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }

        if let Err(e) = buffers.video.remove(buff_start, buff_end) {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }

        self.state = MachineState::Load;
    }

    /// Switch source buffer codec then load initialization segment.
    fn switch_quality(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Switching Quality");

        let buffers = self.media_buffers.as_ref().unwrap();

        let track = match buffers.tracks.get(self.level) {
            Some(track) => track,
            None => return,
        };

        if let Err(e) = buffers.video.change_type(&track.codec) {
            ConsoleService::error(&format!("{:?}", e));
            return;
        }

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!(
            "Level {} Name {} Codec {} Bandwidth {}",
            self.level, track.name, track.codec, track.bandwidth
        ));

        let cid = track.initialization_segment.link;

        self.state = MachineState::Load;

        let cb = self.link.callback_once(Msg::AppendVideo);
        let client = self.ipfs.clone();

        spawn_local(async move { cb.emit(client.cid_cat(cid).await) });
    }

    /// Append audio and video segments to the buffers.
    fn append_buffers(&self, response: Result<(Vec<u8>, Vec<u8>)>) {
        let (mut aud_seg, mut vid_seg) = match response {
            Ok((a, v)) => (a, v),
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let buffers = self.media_buffers.as_ref().unwrap();

        if let Err(e) = buffers.audio.append_buffer_with_u8_array(&mut aud_seg) {
            ConsoleService::warn(&format!("{:#?}", e));
        }

        if let Err(e) = buffers.video.append_buffer_with_u8_array(&mut vid_seg) {
            ConsoleService::warn(&format!("{:#?}", e));
        }
    }

    /// Append video segments to the buffer.
    fn append_video_buffer(&self, response: Result<Vec<u8>>) {
        let mut vid_seg = match response {
            Ok(d) => d,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        let buffers = self.media_buffers.as_ref().unwrap();

        if let Err(e) = buffers.video.append_buffer_with_u8_array(&mut vid_seg) {
            ConsoleService::warn(&format!("{:#?}", e));
        }
    }
}

/// Translate total number of seconds to timecode.
pub fn seconds_to_timecode(seconds: f64) -> (u8, u8, u8) {
    let rem_seconds = seconds.round();

    let hours = (rem_seconds / 3600.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(3600.0);

    let minutes = (rem_seconds / 60.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(60.0);

    let seconds = rem_seconds as u8;

    (hours, minutes, seconds)
}
