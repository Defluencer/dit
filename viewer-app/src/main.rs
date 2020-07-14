use cid::Cid;
use futures_util::future::join;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use ipfs_api::IpfsClient;
use multibase::Base;
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::stream::StreamExt;

const SERVER_PORT: u16 = 3000;
const REQUEST_URI_PATH: &str = "/live/playlist.m3u8";

const PUBSUB_TOPIC_VIDEO: &str = "live_like_video";

struct Playlist {
    version: u8,
    target_duration: u8,
    media_sequence: u32,

    max_seq: u8,

    sequences: VecDeque<Cid>,
}

impl Playlist {
    fn new() -> Self {
        Self {
            version: 3,
            target_duration: 4,
            media_sequence: 0,

            max_seq: 5,

            sequences: VecDeque::with_capacity(5),
        }
    }

    fn add_segment(&mut self, cid: Cid) -> Option<Cid> {
        if self.sequences.len() < self.max_seq as usize {
            self.sequences.push_back(cid);

            return None;
        }

        let result = self.sequences.pop_front();

        self.media_sequence += 1;

        self.sequences.push_back(cid);

        result
    }

    fn to_str(&self) -> String {
        let mut result = format!(
            "#EXTM3U
#EXT-X-VERSION:{ver}
#EXT-X-TARGETDURATION:{dur}
#EXT-X-MEDIA-SEQUENCE:{seq}",
            ver = self.version,
            dur = self.target_duration,
            seq = self.media_sequence,
        );

        for i in 0..self.max_seq as usize {
            match self.sequences.get(i) {
                Some(cid) => {
                    let cid = cid
                        .to_string_of_base(Base::Base58Btc)
                        .expect("Can't get string from cid");

                    let segment = format!(
                        "
#EXTINF:4.000000,
http://localhost:8080/ipfs/{}",
                        cid
                    );

                    result.push_str(&segment);
                }
                None => break,
            }
        }

        result
    }
}

async fn get_file(
    req: Request<Body>,
    data: Arc<RwLock<Playlist>>,
) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());

    if let (&Method::GET, REQUEST_URI_PATH) = (req.method(), req.uri().path()) {
        match data.read() {
            Ok(playlist) => *response.body_mut() = Body::from(playlist.to_str()),
            Err(e) => {
                eprintln!("Lock poisoned. {}", e);
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    } else {
        *response.status_mut() = StatusCode::NOT_FOUND;
    }

    Ok(response)
}

async fn pubsub_sub(playlist: Arc<RwLock<Playlist>>) {
    let client = IpfsClient::default();

    let mut stream = client.pubsub_sub(PUBSUB_TOPIC_VIDEO, true);

    println!("Initialization Complete!");

    while let Some(result) = stream.next().await {
        if let Ok(response) = result {
            //println!("{:#?}", response);

            //TODO match sender id VS streamer is
            /* let sender = match response.from {
                Some(sender) => {
                    let decoded = match Base::decode(&Base::Base64Pad, sender) {
                        Ok(result) => result,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            continue;
                        }
                    };

                    match String::from_utf8(decoded) {
                        Ok(result) => result,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            continue;
                        }
                    }
                }
                None => {
                    eprintln!("No Sender");
                    continue;
                }
            }; */

            let encoded = match response.data {
                Some(data) => data,
                None => {
                    eprintln!("No Data");
                    continue;
                }
            };

            let decoded = match Base::decode(&Base::Base64Pad, encoded) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Can't decode data. {}", e);
                    continue;
                }
            };

            let cid_v0 = match str::from_utf8(&decoded) {
                Ok(cid) => cid,
                Err(e) => {
                    eprintln!("Invalid UTF-8 {}", e);
                    continue;
                }
            };

            println!("CID: {}", cid_v0);

            if let Err(e) = client.pin_add(&cid_v0, true).await {
                eprintln!("Can't pin cid. {}", e);
            }

            let cid_v0 = match Cid::from_str(cid_v0) {
                Ok(cid) => cid,
                Err(e) => {
                    eprintln!("Can't get cid from str. {}", e);
                    continue;
                }
            };

            match playlist.write() {
                Ok(mut playlist) => {
                    if let Some(extra) = playlist.add_segment(cid_v0) {
                        let cid = extra
                            .to_string_of_base(Base::Base58Btc)
                            .expect("Can't get string from cid");

                        if let Err(e) = client.pin_rm(&cid, true).await {
                            eprintln!("Can't unpin cid. {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Lock poisoned. {}", e);
                    return;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Viewer Application Initialization...");

    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

    let playlist = Arc::new(RwLock::new(Playlist::new()));

    let playlist_clone = playlist.clone();

    let make_service = make_service_fn(move |_| {
        //TODO try understand this mess...

        let playlist_clone = playlist_clone.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let playlist_clone = playlist_clone.clone();

                async move { get_file(req, playlist_clone).await }
            }))
        }
    });

    let server = Server::bind(&server_addr).serve(make_service);

    let (_, result) = join(pubsub_sub(playlist), server).await;

    result
}

#[cfg(test)]
mod tests {
    use multibase::Base;

    #[test]
    fn decode_base64pad() {
        let input = "QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD";

        println!("Input Message: {:#?}", input);

        let encoded = Base::encode(&Base::Base64Pad, input);

        println!("Encoded Message: {:#?}", encoded);

        let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Error: ");

        let output = std::str::from_utf8(&decoded).expect("Error: ");

        println!("Output Message: {:#?}", output);

        assert_eq!(input, output);
    }

    use cid::Cid;
    use std::str::FromStr;

    #[test]
    fn encode_cids() {
        let input = "QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD";

        println!("Input Message: {:#?}", input);

        let encoded = Cid::from_str(input).expect("Can't get cid from str");

        println!("Encoded Message: {:#?}", encoded);

        let decoded = encoded.to_string_of_base(Base::Base58Btc).expect("Error: ");

        let output = &decoded;

        println!("Output Message: {:#?}", output);

        assert_eq!(input, output);
    }

    use super::Playlist;

    #[test]
    fn playlist_formatting() {
        let mut playlist = Playlist::new();

        let cid = Cid::from_str("QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let cid = Cid::from_str("QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD")
            .expect("Can't get cid from str");
        playlist.add_segment(cid);

        let output = playlist.to_str();

        println!("{}", output);

        assert_eq!(
            "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:4
#EXT-X-MEDIA-SEQUENCE:5
#EXTINF:4.000000,
http://localhost:8080/ipfs/QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD
#EXTINF:4.000000,
http://localhost:8080/ipfs/QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD
#EXTINF:4.000000,
http://localhost:8080/ipfs/QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD
#EXTINF:4.000000,
http://localhost:8080/ipfs/QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD
#EXTINF:4.000000,
http://localhost:8080/ipfs/QmQrj21qtpNyx5hH8YTWMMja3Tuhwd4Y6XUmk3V6UJ5rhD",
            &output
        );
    }
}
