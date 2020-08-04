use m3u8_rs::playlist::{MasterPlaylist, MediaPlaylist, VariantStream};

use crate::server::SERVER_PORT;
use crate::services::{PATH_1080_60, PATH_480_30, PATH_720_30, PATH_720_60};

pub const HLS_LIST_SIZE: usize = 5;

pub struct Playlists {
    pub master: MasterPlaylist,

    pub playlist_1080_60: MediaPlaylist,
    pub playlist_720_60: MediaPlaylist,
    pub playlist_720_30: MediaPlaylist,
    pub playlist_480_30: MediaPlaylist,
}

impl Playlists {
    pub fn new() -> Self {
        let version = 6;

        let is_i_frame = false;

        let independent_segments = true;

        let variant_1080_60 = VariantStream {
            is_i_frame,
            uri: format!("http://[::1]:{}{}", SERVER_PORT, PATH_1080_60),
            bandwidth: "6811200".to_string(),
            average_bandwidth: Some("6000000".to_string()),
            codecs: Some("avc1.42c02a,mp4a.40.2".to_string()),
            resolution: Some("1920x1080".to_string()),
            frame_rate: Some("60.0".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let variant_720_60 = VariantStream {
            is_i_frame,
            uri: format!("http://[::1]:{}{}", SERVER_PORT, PATH_720_60),
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
            uri: format!("http://[::1]:{}{}", SERVER_PORT, PATH_720_30),
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
            uri: format!("http://[::1]:{}{}", SERVER_PORT, PATH_480_30),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use m3u8_rs::playlist::MediaSegment;

    #[test]
    fn media_playlist_write() {
        let mut playlist = MediaPlaylist {
            version: 4,
            target_duration: 4.0,
            media_sequence: 15,
            segments: Vec::with_capacity(5),
            discontinuity_sequence: 0,
            end_list: false,
            playlist_type: None,
            i_frames_only: false,
            start: None,
            independent_segments: false,
        };

        let segment = MediaSegment {
            uri: format!("http://{}.ipfs.localhost:8080", "HashPlaceHolderHash"),
            duration: 4.000001,
            title: None,
            byte_range: None,
            discontinuity: false,
            key: None,
            map: None,
            program_date_time: None,
            daterange: None,
        };

        playlist.segments.push(segment.clone());
        playlist.segments.push(segment.clone());
        playlist.segments.push(segment.clone());
        playlist.segments.push(segment.clone());
        playlist.segments.push(segment);

        let mut buf: Vec<u8> = Vec::new();

        playlist.write_to(&mut buf).expect("Can't write to buffer");

        let string = String::from_utf8(buf).expect("Invalid UTF8");

        println!("{}", string);
    }

    #[test]
    fn master_playlist_write() {
        let mut master = MasterPlaylist {
            version: 6,
            variants: Vec::with_capacity(1),
            session_data: None,
            session_key: None,
            start: None,
            independent_segments: true,
        };

        let variant = VariantStream {
            is_i_frame: false,
            uri: format!("http://localhost:{}{}", SERVER_PORT, PATH_480_30),
            bandwidth: "2207425".to_string(),
            average_bandwidth: None,
            codecs: Some("avc1.64001f,mp4a.40.2".to_string()),
            resolution: Some("1920x1080".to_string()),
            frame_rate: Some("60.0".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        master.variants.push(variant);

        let mut buf: Vec<u8> = Vec::new();

        master.write_to(&mut buf).expect("Can't write to buffer");

        let string = String::from_utf8(buf).expect("Invalid UTF8");

        println!("{}", string);
    }
}
