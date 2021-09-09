use std::collections::VecDeque;
use std::rc::Rc;
use std::str;
use std::str::FromStr;

use crate::utils::seconds_to_timecode;
use crate::utils::{ExponentialMovingAverage, IpfsService};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use web_sys::{HtmlMediaElement, MediaSource, MediaSourceReadyState, SourceBuffer, Url};

use yew::prelude::{classes, html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::services::ConsoleService;

use linked_data::beacon::Beacon;
use linked_data::video::{SetupNode, Track, VideoMetadata};

use either::Either;

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
    beacon: Rc<Beacon>,

    buffer: VecDeque<Cid>,

    drop_sig: Rc<()>,
}

/// Video player for live streams and on demand.
pub struct VideoPlayer {
    link: ComponentLink<Self>,

    ipfs: IpfsService,

    player_type: Either<LiveStream, Rc<VideoMetadata>>,

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
    PubSub(Result<(String, Vec<u8>)>),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub ipfs: IpfsService,
    pub beacon_or_metadata: Either<Rc<Beacon>, Rc<VideoMetadata>>,
}

impl Component for VideoPlayer {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let Props {
            ipfs,
            beacon_or_metadata,
        } = props;

        let ema = ExponentialMovingAverage::new();

        let media_source = match MediaSource::new() {
            Ok(media_source) => media_source,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                std::process::abort();
            }
        };

        let object_url = match Url::create_object_url_with_source(&media_source) {
            Ok(object_url) => object_url,
            Err(e) => {
                ConsoleService::error(&format!("{:#?}", e));
                std::process::abort();
            }
        };

        let cb = link.callback(|_| Msg::SourceOpen);
        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
        media_source.set_onsourceopen(Some(closure.as_ref().unchecked_ref()));
        let source_open_closure = Some(closure);

        let mut poster_link = String::from("ipfs://");

        let player_type = match beacon_or_metadata {
            Either::Left(beacon) => {
                let live = LiveStream {
                    beacon,
                    buffer: VecDeque::with_capacity(5),
                    drop_sig: Rc::from(()),
                };

                //TODO default thumbnail image
                poster_link.push_str("bafkreicovb5qdvrine4vidt77xahhvovahmekvsojbiqewp7ih7pzvnn7i");

                Either::Left(live)
            }
            Either::Right(metadata) => {
                poster_link.push_str(&metadata.image.link.to_string());

                Either::Right(metadata)
            }
        };

        let comp = Self {
            link,

            ipfs,
            player_type,

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
        };

        match comp.player_type {
            Either::Left(_) => comp.subscribe(),
            _ => {}
        }

        comp
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

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let live = match &mut self.player_type {
            Either::Left(live) => live,
            _ => return false,
        };

        let beacon = match props.beacon_or_metadata {
            Either::Left(beacon) => beacon,
            _ => return false,
        };

        if Rc::ptr_eq(&live.beacon, &beacon) {
            return false;
        }

        live.beacon = beacon;
        live.drop_sig = Rc::from(());

        self.subscribe();

        #[cfg(debug_assertions)]
        ConsoleService::info("Video Player Changed");

        true
    }

    fn view(&self) -> Html {
        html! {
            <ybc::Image size=ybc::ImageSize::Is16by9>
                <video class=classes!("has-ratio") width=640 height=360 id="video_player" autoplay="true" controls=true poster=self.poster_link.clone() />
            </ybc::Image>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let window = match web_sys::window() {
                Some(window) => window,
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Window Object");
                    return;
                }
            };

            let document = match window.document() {
                Some(document) => document,
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Document Object");
                    return;
                }
            };

            let element = match document.get_element_by_id("video_player") {
                Some(document) => document,
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Element by Id");
                    return;
                }
            };

            let media_element: HtmlMediaElement = match element.dyn_into() {
                Ok(document) => document,
                Err(e) => {
                    ConsoleService::error(&format!("{:#?}", e));
                    return;
                }
            };

            media_element.set_src(&self.object_url);

            self.seeking_closure = match self.player_type {
                Either::Right(_) => {
                    let cb = self.link.callback(|_| Msg::Seeking);
                    let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
                    media_element.set_onseeking(Some(closure.as_ref().unchecked_ref()));

                    Some(closure)
                }
                _ => None,
            };

            self.media_element = Some(media_element);
        }
    }

    fn destroy(&mut self) {
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Window Object");
                return;
            }
        };

        if self.handle != 0 {
            window.clear_timeout_with_handle(self.handle);
        }
    }
}

impl VideoPlayer {
    fn subscribe(&self) {
        let live = match &self.player_type {
            Either::Left(live) => live,
            _ => return,
        };

        let topic = live.beacon.topics.video.clone();

        if topic.is_empty() {
            return;
        }

        spawn_local({
            let ipfs = self.ipfs.clone();
            let cb = self.link.callback(Msg::PubSub);
            let sig = live.drop_sig.clone();

            async move { ipfs.pubsub_sub(topic, cb, sig).await }
        });
    }

    /// Callback when MediaSource is linked to video element.
    fn on_source_open(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("On Source Open");

        self.media_source.set_onsourceopen(None);
        self.source_open_closure = None;

        if let Either::Right(metadata) = &self.player_type {
            self.media_source.set_duration(metadata.duration);

            spawn_local({
                let cb = self.link.callback_once(Msg::SetupNode);
                let ipfs = self.ipfs.clone();
                let cid = metadata.video.link;

                async move { cb.emit(ipfs.dag_get(cid, Some(SETUP_PATH)).await) }
            });
        }
    }

    /// Callback when GossipSub receive an update.
    fn on_pubsub_update(&mut self, result: Result<(String, Vec<u8>)>) {
        let res = match result {
            Ok(res) => res,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("PubSub Message Received");

        let (from, data) = res;

        let live = match &mut self.player_type {
            Either::Left(live) => live,
            _ => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Live Stream");
                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        if from != live.beacon.peer_id {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Unauthorized Sender");
            return;
        }

        let data = match str::from_utf8(&data) {
            Ok(data) => data,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Message => {}", data));

        let cid = match Cid::from_str(data) {
            Ok(cid) => cid,
            Err(e) => {
                ConsoleService::error(&format!("{:?}", e));
                return;
            }
        };

        live.buffer.push_back(cid);

        if self.media_buffers.is_none() {
            spawn_local({
                let cb = self.link.callback_once(Msg::SetupNode);
                let ipfs = self.ipfs.clone();

                async move { cb.emit(ipfs.dag_get(cid, Some("/setup/")).await) }
            });
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

        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Window Object");
                return;
            }
        };

        match window.set_timeout_with_callback_and_timeout_and_arguments_0(
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

        for track in setup_node.tracks.iter() {
            if !MediaSource::is_type_supported(&track.codec) {
                ConsoleService::error(&format!("MIME Type {:?} unsupported", &track.codec));
                continue;
            }

            #[cfg(debug_assertions)]
            ConsoleService::info(&format!(
                "Name {} Codec {} Bandwidth {}",
                track.name, track.codec, track.bandwidth
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

        let audio = match audio_buffer {
            Some(audio) => audio,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Audio Buffer");
                return;
            }
        };

        let video = match video_buffer {
            Some(video) => video,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Video Buffer");
                return;
            }
        };

        let media_buffer = MediaBuffers {
            audio,
            video,
            tracks: setup_node.tracks,
        };

        let cb = self.link.callback(|_| Msg::UpdateEnd);
        let closure = Closure::wrap(Box::new(move || cb.emit(())) as Box<dyn Fn()>);
        media_buffer
            .video
            .set_onupdateend(Some(closure.as_ref().unchecked_ref()));

        self.update_end_closure = Some(closure);

        let audio_path = match media_buffer.tracks.get(0) {
            Some(track) => track.initialization_segment.link.to_string(),
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Track Index 0");
                return;
            }
        };

        let video_path = match media_buffer.tracks.get(1) {
            Some(track) => track.initialization_segment.link.to_string(),
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Track Index 1");
                return;
            }
        };

        self.media_buffers = Some(media_buffer);
        self.state = MachineState::Load;

        spawn_local({
            let cb = self.link.callback_once(Msg::Append);
            let ipfs = self.ipfs.clone();

            async move { cb.emit(ipfs.double_path_cat(audio_path, video_path).await) }
        });
    }

    /// Load either live or VOD segment.
    fn load_segment(&mut self) {
        match self.player_type {
            Either::Right(_) => self.load_vod_segment(),
            Either::Left(_) => self.load_live_segment(),
        }
    }

    /// Try get cid from live buffer then fetch video data from ipfs.
    fn load_live_segment(&mut self) {
        let live = match &mut self.player_type {
            Either::Left(live) => live,
            _ => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Live Stream");
                return;
            }
        };

        let cid_string = match live.buffer.pop_front() {
            Some(cid) => cid.to_string(),
            None => return self.set_timeout(),
        };

        #[cfg(debug_assertions)]
        ConsoleService::info("Loading Live Media Segments");

        let track_name = match self.media_buffers.as_ref() {
            Some(buf) => match buf.tracks.get(self.level) {
                Some(track) => &track.name,
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Track");
                    return;
                }
            },
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

        let audio_path = format!("{}/track/audio", cid_string);
        let video_path = format!("{}/track/{}", cid_string, track_name);

        self.state = MachineState::AdaptativeBitrate;
        self.ema.start_timer();

        spawn_local({
            let cb = self.link.callback_once(Msg::Append);
            let ipfs = self.ipfs.clone();

            async move { cb.emit(ipfs.double_path_cat(audio_path, video_path).await) }
        });
    }

    /// Get CID from timecode then fetch video data from ipfs.
    fn load_vod_segment(&mut self) {
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

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

        let cid_string = match &self.player_type {
            Either::Right(metadata) => metadata.video.link.to_string(),
            _ => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Metadata");
                return;
            }
        };

        let audio_path = format!(
            "{}/time/hour/{}/minute/{}/second/{}/video/track/audio",
            cid_string, hours, minutes, seconds,
        );

        let track_name = match buffers.tracks.get(self.level) {
            Some(track) => &track.name,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Track");
                return;
            }
        };

        let video_path = format!(
            "{}/time/hour/{}/minute/{}/second/{}/video/track/{}",
            cid_string, hours, minutes, seconds, track_name,
        );

        self.state = MachineState::AdaptativeBitrate;
        self.ema.start_timer();

        spawn_local({
            let cb = self.link.callback_once(Msg::Append);
            let ipfs = self.ipfs.clone();

            async move { cb.emit(ipfs.double_path_cat(audio_path, video_path).await) }
        });
    }

    /// Recalculate download speed then set quality level.
    fn check_abr(&mut self) {
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

        let bandwidth = match buffers.tracks.get(self.level) {
            Some(track) => track.bandwidth as f64,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Track");
                return;
            }
        };

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
        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

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

            match self.media_element.as_ref() {
                Some(media_element) => media_element.set_current_time(new_time),
                None => {
                    #[cfg(debug_assertions)]
                    ConsoleService::error("No Media Element");
                    return;
                }
            }
        }

        if current_time > buff_start + BACK_BUFFER_LENGTH {
            #[cfg(debug_assertions)]
            ConsoleService::info("Back Buffer Full");
            return self.flush_buffer();
        }

        if let Either::Right(metadata) = &self.player_type {
            if buff_end >= metadata.duration {
                #[cfg(debug_assertions)]
                ConsoleService::info("End Of Video");
                return;
            }

            if current_time + FORWARD_BUFFER_LENGTH < buff_end {
                #[cfg(debug_assertions)]
                ConsoleService::info("Forward Buffer Full");
                return self.set_timeout();
            }
        }

        self.load_segment()
    }

    /// Flush everything or just back buffer.
    fn flush_buffer(&mut self) {
        #[cfg(debug_assertions)]
        ConsoleService::info("Flushing Buffer");

        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

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
                ConsoleService::error("No Media Element");
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

        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

        let track = match buffers.tracks.get(self.level) {
            Some(track) => track,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Track");
                return;
            }
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

        spawn_local({
            let cb = self.link.callback_once(Msg::AppendVideo);
            let ipfs = self.ipfs.clone();

            async move { cb.emit(ipfs.cid_cat(cid).await) }
        });
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

        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

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

        let buffers = match self.media_buffers.as_ref() {
            Some(buf) => buf,
            None => {
                #[cfg(debug_assertions)]
                ConsoleService::error("No Media Buffers");
                return;
            }
        };

        if let Err(e) = buffers.video.append_buffer_with_u8_array(&mut vid_seg) {
            ConsoleService::warn(&format!("{:#?}", e));
        }
    }
}
