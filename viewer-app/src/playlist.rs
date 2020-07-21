use m3u8_rs::playlist::{MasterPlaylist, MediaPlaylist, VariantStream};

use crate::server::SERVER_PORT;
use crate::services::{
    REQUEST_URI_PATH_1080_60, REQUEST_URI_PATH_480_30, REQUEST_URI_PATH_720_30,
    REQUEST_URI_PATH_720_60,
};

pub struct Playlists {
    pub master: MasterPlaylist,

    pub playlist_1080_60: MediaPlaylist,
    pub playlist_720_60: MediaPlaylist,
    pub playlist_720_30: MediaPlaylist,
    pub playlist_480_30: MediaPlaylist,
}

impl Playlists {
    pub fn new() -> Self {
        let version = 4;
        let independent_segments = true;

        let variant_1080_60 = VariantStream {
            is_i_frame: true,
            uri: format!(
                "http://localhost:{}{}",
                SERVER_PORT, REQUEST_URI_PATH_1080_60
            ),
            bandwidth: "6000000".to_string(),
            average_bandwidth: None,
            codecs: Some("avc1.42e00a,mp4a.40.2".to_string()),
            resolution: Some("1920x1080".to_string()),
            frame_rate: Some("60".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let variant_720_60 = VariantStream {
            is_i_frame: true,
            uri: format!(
                "http://localhost:{}{}",
                SERVER_PORT, REQUEST_URI_PATH_720_60
            ),
            bandwidth: "4500000".to_string(),
            average_bandwidth: None,
            codecs: Some("avc1.42e00a,mp4a.40.2".to_string()),
            resolution: Some("1280x720".to_string()),
            frame_rate: Some("60".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let variant_720_30 = VariantStream {
            is_i_frame: true,
            uri: format!(
                "http://localhost:{}{}",
                SERVER_PORT, REQUEST_URI_PATH_720_30
            ),
            bandwidth: "3000000".to_string(),
            average_bandwidth: None,
            codecs: Some("avc1.42e00a,mp4a.40.2".to_string()),
            resolution: Some("1280x720".to_string()),
            frame_rate: Some("30".to_string()),
            audio: None,
            video: None,
            subtitles: None,
            closed_captions: None,
            alternatives: vec![],
        };

        let variant_480_30 = VariantStream {
            is_i_frame: true,
            uri: format!(
                "http://localhost:{}{}",
                SERVER_PORT, REQUEST_URI_PATH_480_30
            ),
            bandwidth: "2000000".to_string(),
            average_bandwidth: None,
            codecs: Some("avc1.42e00a,mp4a.40.2".to_string()),
            resolution: Some("1920x1080".to_string()),
            frame_rate: Some("60".to_string()),
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
            segments: Vec::with_capacity(5),
            discontinuity_sequence: 0,
            end_list: false,
            playlist_type: None,
            i_frames_only: true,
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

/* pub struct Playlists {
    pub master: MasterPlaylist,

    pub media: HashMap<StreamVariant, MediaPlaylist>,
} */

/* impl MediaPlaylist {
    pub fn add_segment(&mut self, variant: &StreamVariant, cid: Cid) -> Option<Cid> {
        let sequences = self
            .sequences
            .get_mut(variant)
            .expect("Sequences not initialized");

        if sequences.len() < self.hls_list_size {
            sequences.push_back(cid);

            return None;
        }

        let result = sequences.pop_front();

        self.media_sequence += 1;

        sequences.push_back(cid);

        result
    }

    pub fn get_media_playlist(&self, variant: &StreamVariant) -> String {
        let mut result = format!(
            "#EXTM3U
#EXT-X-VERSION:{ver}
#EXT-X-TARGETDURATION:{dur}
#EXT-X-MEDIA-SEQUENCE:{seq}",
            ver = self.version,
            dur = self.target_duration,
            seq = self.media_sequence,
        );

        let sequences = self
            .sequences
            .get(variant)
            .expect("Sequences not initialized");

        for i in 0..self.hls_list_size {
            match sequences.get(i) {
                Some(cid) => {
                    let segment = format!(
                        "
#EXTINF:{number:.prec$},
http://{cid}.ipfs.localhost:8080",
                        number = self.target_duration as f32,
                        prec = 6,
                        cid = cid.to_string()
                    );

                    result.push_str(&segment);
                }
                None => break,
            }
        }

        result
    }
}

pub fn get_master_playlist() -> String {
    let master = format!(
        r#"#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=6000000,RESOLUTION=1920x1080,FRAME-RATE=60,CODECS="avc1.42e00a,mp4a.40.2"
http://localhost:{PORT}{PATH1}
#EXT-X-STREAM-INF:BANDWIDTH=4500000,RESOLUTION=1280x720,FRAME-RATE=60,CODECS="avc1.42e00a,mp4a.40.2"
http://localhost:{PORT}{PATH2}
#EXT-X-STREAM-INF:BANDWIDTH=3000000,RESOLUTION=1280x720,FRAME-RATE=30,CODECS="avc1.42e00a,mp4a.40.2"
http://localhost:{PORT}{PATH3}
#EXT-X-STREAM-INF:BANDWIDTH=2000000,RESOLUTION=840x480,FRAME-RATE=30,CODECS="avc1.42e00a,mp4a.40.2"
http://localhost:{PORT}{PATH4}"#,
        PORT = SERVER_PORT,
        PATH1 = REQUEST_URI_PATH_1080_60,
        PATH2 = REQUEST_URI_PATH_720_60,
        PATH3 = REQUEST_URI_PATH_720_30,
        PATH4 = REQUEST_URI_PATH_480_30,
    );

    master
} */

/* #[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn playlist_formatting() {
        let mut playlist = MediaPlaylist::new(3, 4, 5);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(&StreamVariant::High1080p60, cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(&StreamVariant::High1080p60, cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(&StreamVariant::High1080p60, cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(&StreamVariant::High1080p60, cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(&StreamVariant::High1080p60, cid);

        let output = playlist.get_media_playlist(&StreamVariant::High1080p60);

        println!("{}", output);

        assert_eq!(
            "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:4
#EXT-X-MEDIA-SEQUENCE:0
#EXTINF:4.000000,
http://bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4.ipfs.localhost:8080
#EXTINF:4.000000,
http://bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4.ipfs.localhost:8080
#EXTINF:4.000000,
http://bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4.ipfs.localhost:8080
#EXTINF:4.000000,
http://bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4.ipfs.localhost:8080
#EXTINF:4.000000,
http://bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4.ipfs.localhost:8080",
            &output
        );
    }
} */
