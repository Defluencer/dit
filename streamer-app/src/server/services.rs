use crate::actors::VideoData;

use std::convert::TryFrom;
use std::fmt::Debug;
use std::path::Path;

use futures_util::stream::TryStreamExt;

use tokio::sync::mpsc::UnboundedSender;
use tokio_util::io::StreamReader;

use hyper::header::{HeaderValue, LOCATION};
use hyper::{Body, Error, Method, Request, Response, StatusCode};

use ipfs_api::IpfsClient;

use cid::Cid;

use m3u8_rs::playlist::Playlist;

const M3U8: &str = "m3u8";
pub const MP4: &str = "mp4";
pub const FMP4: &str = "fmp4";

const OPTIONS: ipfs_api::request::Add = ipfs_api::request::Add {
    trickle: None,
    only_hash: None,
    wrap_with_directory: None,
    chunker: None,
    pin: Some(false),
    raw_leaves: None,
    cid_version: Some(1),
    hash: None,
    inline: None,
    inline_limit: None,
};

pub async fn put_requests(
    req: Request<Body>,
    collector: UnboundedSender<VideoData>,
    ipfs: IpfsClient,
) -> Result<Response<Body>, Error> {
    #[cfg(debug_assertions)]
    println!("Service: {:#?}", req);

    let mut res = Response::new(Body::empty());

    let (parts, body) = req.into_parts();

    let path = Path::new(parts.uri.path());

    if parts.method != Method::PUT
        || path.extension() == None
        || (path.extension().unwrap() != M3U8
            && path.extension().unwrap() != FMP4
            && path.extension().unwrap() != MP4)
    {
        return not_found_response(res);
    }

    if path.extension().unwrap() == M3U8 {
        return manifest_response(res, body, &path, collector).await;
    }

    //Change error type
    let stream =
        body.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()));

    //Stream to AsyncRead
    let reader = StreamReader::new(stream);

    let cid = match ipfs.add_with_options(reader, OPTIONS).await {
        Ok(res) => Cid::try_from(res.hash).expect("Invalid Cid"),
        Err(error) => return internal_error_response(res, &error),
    };

    #[cfg(debug_assertions)]
    println!("IPFS: add => {}", &cid.to_string());

    let msg = VideoData::Segment((path.to_path_buf(), cid));

    if let Err(error) = collector.send(msg) {
        return internal_error_response(res, &error);
    }

    *res.status_mut() = StatusCode::CREATED;

    let header_value = HeaderValue::from_str(parts.uri.path()).expect("Invalid Header Value");

    res.headers_mut().insert(LOCATION, header_value);

    #[cfg(debug_assertions)]
    println!("Service: {:#?}", res);

    Ok(res)
}

fn not_found_response(mut res: Response<Body>) -> Result<Response<Body>, Error> {
    *res.status_mut() = StatusCode::NOT_FOUND;

    #[cfg(debug_assertions)]
    println!("Service: {:#?}", res);

    Ok(res)
}

async fn manifest_response(
    mut res: Response<Body>,
    body: Body,
    path: &Path,
    collector: UnboundedSender<VideoData>,
) -> Result<Response<Body>, Error> {
    let bytes = hyper::body::to_bytes(body).await?;

    let playlist = match m3u8_rs::parse_playlist(&bytes) {
        Ok((_, playlist)) => playlist,
        Err(e) => return internal_error_response(res, &e),
    };

    if let Playlist::MasterPlaylist(playlist) = playlist {
        let msg = VideoData::Playlist(playlist);

        if let Err(error) = collector.send(msg) {
            return internal_error_response(res, &error);
        }
    }

    *res.status_mut() = StatusCode::NO_CONTENT;

    let header_value = HeaderValue::from_str(path.to_str().unwrap()).unwrap();

    res.headers_mut().insert(LOCATION, header_value);

    #[cfg(debug_assertions)]
    println!("Service: {:#?}", res);

    Ok(res)
}

fn internal_error_response(
    mut res: Response<Body>,
    error: &dyn Debug,
) -> Result<Response<Body>, Error> {
    eprintln!("Service Error: {:#?}", error);

    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

    #[cfg(debug_assertions)]
    println!("Service: {:#?}", res);

    Ok(res)
}
