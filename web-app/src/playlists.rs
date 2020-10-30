use std::convert::TryFrom;
use url::Url;

use yew::services::ConsoleService;

use m3u8_rs::playlist::{MasterPlaylist, MediaPlaylist, MediaSegment, VariantStream};

use cid::Cid;

// Hard-Coded for now...
pub const PATH_MASTER: &str = "/livelike/master.m3u8";
pub const PATH_1080_60: &str = "/livelike/1080p60/index.m3u8";
pub const PATH_720_60: &str = "/livelike/720p60/index.m3u8";
pub const PATH_720_30: &str = "/livelike/720p30/index.m3u8";
pub const PATH_480_30: &str = "/livelike/480p30/index.m3u8";

pub const HLS_LIST_SIZE: usize = 5;

pub const STREAMER_PEER_ID: &str = "12D3KooWAPZ3QZnZUJw3BgEX9F7XL383xFNiKQ5YKANiRC3NWvpo";

pub struct Playlists {
    master: MasterPlaylist,

    playlist_1080_60: MediaPlaylist,
    playlist_720_60: MediaPlaylist,
    playlist_720_30: MediaPlaylist,
    playlist_480_30: MediaPlaylist,

    load: bool,
}

impl Playlists {
    pub fn new() -> Self {
        let version = 6;

        let is_i_frame = false;

        let independent_segments = true;

        let variant_1080_60 = VariantStream {
            is_i_frame,
            uri: PATH_1080_60.into(),
            bandwidth: "6811200".into(),
            average_bandwidth: Some("6000000".into()),
            codecs: Some("avc1.42c02a,mp4a.40.2".into()),
            resolution: Some("1920x1080".into()),
            frame_rate: Some("60.0".into()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let variant_720_60 = VariantStream {
            is_i_frame,
            uri: PATH_720_60.into(),
            bandwidth: "5161200".to_string(),
            average_bandwidth: Some("4500000".to_string()),
            codecs: Some("avc1.42c020,mp4a.40.2".to_string()),
            resolution: Some("1280x720".to_string()),
            frame_rate: Some("60.0".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let variant_720_30 = VariantStream {
            is_i_frame,
            uri: PATH_720_30.into(),
            bandwidth: "3511200".to_string(),
            average_bandwidth: Some("3000000".to_string()),
            codecs: Some("avc1.42c01f,mp4a.40.2".to_string()),
            resolution: Some("1280x720".to_string()),
            frame_rate: Some("30.0".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let variant_480_30 = VariantStream {
            is_i_frame,
            uri: PATH_480_30.into(),
            bandwidth: "2411200".to_string(),
            average_bandwidth: Some("2000000".to_string()),
            codecs: Some("avc1.42c01f,mp4a.40.2".to_string()),
            resolution: Some("854x480".to_string()),
            frame_rate: Some("30.0".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let master = MasterPlaylist {
            version,
            variants: vec![
                variant_1080_60,
                variant_720_60,
                variant_720_30,
                variant_480_30,
            ],
            session_data: None,
            session_key: None,
            start: None,
            independent_segments,
        };

        let playlist = MediaPlaylist {
            version,
            target_duration: 4.0,
            media_sequence: 0,
            segments: Vec::with_capacity(HLS_LIST_SIZE),
            discontinuity_sequence: 0,
            end_list: false,
            playlist_type: None,
            i_frames_only: false,
            start: None,
            independent_segments,
        };

        Self {
            master,

            playlist_1080_60: playlist.clone(),
            playlist_720_60: playlist.clone(),
            playlist_720_30: playlist.clone(),
            playlist_480_30: playlist,

            load: false,
        }
    }

    pub fn get_playlist(&self, url: String) -> String {
        let url = Url::parse(&url).expect("Cannot Parse Url");

        let mut buf: Vec<u8> = Vec::new();

        match url.path() {
            PATH_MASTER => self
                .master
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_1080_60 => self
                .playlist_1080_60
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_720_60 => self
                .playlist_720_60
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_720_30 => self
                .playlist_720_30
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_480_30 => self
                .playlist_480_30
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            _ => return String::from("Playlist Error"),
        };

        String::from_utf8(buf).expect("Invalid UTF-8")
    }

    fn update_playlists(&mut self, video_cid: &Cid) {
        update_playlist(&mut self.playlist_1080_60, video_cid, "1080p60");
        update_playlist(&mut self.playlist_720_60, video_cid, "720p60");
        update_playlist(&mut self.playlist_720_30, video_cid, "720p30");
        update_playlist(&mut self.playlist_480_30, video_cid, "480p30");
    }

    pub fn pubsub_message(&mut self, from: String, data: Vec<u8>) {
        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Sender => {}", from));

        if from != STREAMER_PEER_ID {
            #[cfg(debug_assertions)]
            ConsoleService::warn("Unauthorized Sender");

            return;
        }

        let data_utf8 = match std::str::from_utf8(&data) {
            Ok(string) => string,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::warn("Message Invalid UTF-8");

                return;
            }
        };

        let video_cid = match Cid::try_from(data_utf8) {
            Ok(cid) => cid,
            Err(_) => {
                #[cfg(debug_assertions)]
                ConsoleService::warn("Message Invalid CID");

                return;
            }
        };

        #[cfg(debug_assertions)]
        ConsoleService::info(&format!("Message => {}", video_cid.to_string()));

        self.update_playlists(&video_cid);

        if !self.load {
            self.load = true;

            crate::bindings::start_video();
        }
    }
}

fn update_playlist(playlist: &mut MediaPlaylist, cid: &Cid, quality: &str) {
    let segment = MediaSegment {
        uri: format!("/ipfs/{}/quality/{}", cid.to_string(), quality),
        duration: 4.0,
        title: None,
        byte_range: None,
        discontinuity: false,
        key: None,
        map: None,
        program_date_time: None,
        daterange: None,
    };

    if playlist.segments.len() >= HLS_LIST_SIZE {
        playlist.segments.remove(0);
        playlist.media_sequence += 1;
    }

    playlist.segments.push(segment);
}
