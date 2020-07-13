use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use ipfs_api::IpfsClient;
use multibase::Base;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::stream::StreamExt;

mod subscription;

const PUBSUB_TOPIC_VIDEO: &str = "live_like_video";

const PLAYLIST: &str = "#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:0
#EXT-X-PLAYLIST-TYPE:EVENT
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmThYQy7LFqnAFAkRVM2sTjHDC9Aq41oBQrd5jTmRyA8XR
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmRKm9DKH7L6mJgWPrBR4SQ82YMiQ7yxctqPV8UakyD2ju
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmVc7CdCAh6SY1R7mY9VS2fqHyyGgPo7rK3YJKt7VGhva6
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmXXqXFUpWf8w5Fsg1W7rtTgHhA5diZs5Lo474yRQ5ENWZ
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmS2o2MYvqEKbC2QRyYXzEXU2fYHfWDpjNncUckNv31rCF
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmZ6nVNtgYR5rF5wuwtgVt1Gvd8zuGcuxJRfkwRaktBEeN
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmWwyEavE4zkj199V3TC4kjDBWAfczmrPjgxGoRLNA4XDR
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmeDa3b9SCrbeQxajav1GTEEHrqaZFTJ1PJzHZRWSrnjB8
#EXTINF:2.000000,
http://127.0.0.1:8080/ipfs/QmP2SSfQUXoSMnuPhPjDyCup2XcnvZTcaiRB5gFm4LStYu
#EXTINF:0.166667,
http://127.0.0.1:8080/ipfs/QmWa6L1kq7ccsUmxyoGrY6wnP1sHDyv7wcAHPDiuuUopXe
#EXT-X-ENDLIST";

async fn get_file(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/live/playlist.m3u8") => {
            *response.body_mut() = Body::from(PLAYLIST);
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Ok(response)
}

#[tokio::main]
async fn main() {
    println!("Viewer Application Initialization...");

    let server_addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(get_file)) });

    let server = Server::bind(&server_addr).serve(make_service);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

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

            let mut cid_v0 = String::from_utf8(decoded).expect("Invalid UTF-8");

            let ipfs_path = cid_v0.insert_str(0, "/ipfs/");

            println!("Path: {:#?}", ipfs_path);

            let res = client.pin_add(&cid_v0, true).await;

            //TODO append the playlist with the segment cid and unpin older segment
        }
    }
}

#[cfg(test)]
mod tests {
    use multibase::Base;

    #[test]
    fn transcode() {
        let input = "Hello World!!!";

        println!("Input Message: {:#?}", input);

        let encoded = Base::encode(&Base::Base64Pad, input);

        println!("Encoded Message: {:#?}", encoded);

        let decoded = Base::decode(&Base::Base64Pad, encoded).expect("Error: ");

        let output = std::str::from_utf8(&decoded).expect("Error: ");

        println!("Output Message: {:#?}", output);

        assert_eq!(input, output);
    }
}
