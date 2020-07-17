use std::ffi::OsStr;
use std::io::BufReader;
use std::path::Path;

use futures::TryStreamExt as _;
use hyper::{Body, Error, Method, Request, Response, StatusCode};
use ipfs_api::IpfsClient;

async fn ipfs_add_pub(
    request: Request<Body>,
    client: IpfsClient,
    topic: &str,
) -> Result<Response<Body>, Error> {
    let mut response = Response::new(Body::empty());

    #[cfg(debug_assertions)]
    println!("ipfs_add_pub");

    let body = request.into_body();

    /* let data = match hyper::body::aggregate(request.into_body()).await {
        Ok(data) => Cursor::new(data),
        Err(e) => {
            eprintln!("Can't read request data. {}", e);
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
    }; */

    let add_response = match client.add(body).await {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Can't add data to ipfs. {}", e);
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
    };

    let cid = &add_response.hash;

    if let Err(e) = client.pubsub_pub(topic, cid).await {
        eprintln!("Can't publish message. {}", e);
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(response);
    }

    println!("{:#?} => {:#?}", cid, topic);

    Ok(response)
}

const REQUEST_URI_PATH_1080_60: &str = "/live/1080_60";
const REQUEST_URI_PATH_720_60: &str = "/live/720_60";
const REQUEST_URI_PATH_720_30: &str = "/live/720_30";
const REQUEST_URI_PATH_480_30: &str = "/live/480_30";

const PUBSUB_TOPIC_1080_60: &str = "live_like_video_1080_60";
const PUBSUB_TOPIC_720_60: &str = "live_like_video_720_60";
const PUBSUB_TOPIC_720_30: &str = "live_like_video_720_30";
const PUBSUB_TOPIC_480_30: &str = "live_like_video_480_30";

pub async fn put_requests(req: Request<Body>, client: IpfsClient) -> Result<Response<Body>, Error> {
    let mut res = Response::new(Body::empty());

    if Method::PUT != *req.method() {
        *res.status_mut() = StatusCode::NOT_FOUND;
        return Ok(res);
    }

    let path = Path::new(req.uri().path());

    if path.extension() != Some(OsStr::new("ts")) {
        //Silently ignore .m3u8
        #[cfg(debug_assertions)]
        println!("Ignoring files other than .ts");
        return Ok(res);
    }

    let parent = path.parent().unwrap().to_str().unwrap();

    match parent {
        REQUEST_URI_PATH_1080_60 => ipfs_add_pub(req, client, PUBSUB_TOPIC_1080_60).await,
        REQUEST_URI_PATH_720_60 => ipfs_add_pub(req, client, PUBSUB_TOPIC_720_60).await,
        REQUEST_URI_PATH_720_30 => ipfs_add_pub(req, client, PUBSUB_TOPIC_720_30).await,
        REQUEST_URI_PATH_480_30 => ipfs_add_pub(req, client, PUBSUB_TOPIC_480_30).await,
        _ => {
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_check() {
        let string = "/live/1080_60/1920x1080_60_0.ts";

        let path = Path::new(string);

        assert!(path.starts_with(REQUEST_URI_PATH_1080_60))
    }
}
