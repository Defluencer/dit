use cid::Cid;
use std::collections::VecDeque;

//TODO master playlist

pub struct Playlist {
    version: u8,
    target_duration: u8,
    media_sequence: u32,

    hls_list_size: usize,

    sequences: VecDeque<Cid>,
}

impl Playlist {
    pub fn new(version: u8, target_duration: u8, hls_list_size: usize) -> Self {
        Self {
            version,
            target_duration,
            media_sequence: 0,

            hls_list_size,

            sequences: VecDeque::with_capacity(hls_list_size),
        }
    }

    pub fn add_segment(&mut self, cid: Cid) -> Option<Cid> {
        if self.sequences.len() < self.hls_list_size {
            self.sequences.push_back(cid);

            return None;
        }

        let result = self.sequences.pop_front();

        self.media_sequence += 1;

        self.sequences.push_back(cid);

        result
    }

    pub fn to_str(&self) -> String {
        let mut result = format!(
            "#EXTM3U
#EXT-X-VERSION:{ver}
#EXT-X-TARGETDURATION:{dur}
#EXT-X-MEDIA-SEQUENCE:{seq}",
            ver = self.version,
            dur = self.target_duration,
            seq = self.media_sequence,
        );

        for i in 0..self.hls_list_size {
            match self.sequences.get(i) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn playlist_formatting() {
        let mut playlist = Playlist::new(3, 4, 5);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("bafybeiaeyyvl3kjmelqgo5byyfdqy76xabfgymbpjz7szmnocreo6graw4")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let output = playlist.to_str();

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
}
