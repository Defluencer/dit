use crate::actors::VideoData;

use std::path::Path;

use tokio::sync::mpsc::Sender;

use hyper::header::{HeaderValue, LOCATION};
use hyper::{Body, Error, Method, Request, Response, StatusCode};

const M3U8: &str = "m3u8";
const MP4: &str = "mp4";
const FMP4: &str = "fmp4";

pub async fn put_requests(
    req: Request<Body>,
    collector: Sender<VideoData>,
) -> Result<Response<Body>, Error> {
    #[cfg(debug_assertions)]
    println!("{:#?}", req);

    let mut res = Response::new(Body::empty());

    let (parts, body) = req.into_parts();

    let path = Path::new(parts.uri.path());

    if parts.method != Method::PUT
        || path.extension() == None
        || (path.extension().unwrap() != M3U8
            && path.extension().unwrap() != FMP4
            && path.extension().unwrap() != MP4)
    {
        *res.status_mut() = StatusCode::NOT_FOUND;

        #[cfg(debug_assertions)]
        println!("{:#?}", res);

        return Ok(res);
    }

    if path.extension().unwrap() == M3U8 {
        #[cfg(debug_assertions)]
        {
            let data = hyper::body::to_bytes(body).await?;

            match m3u8_rs::parse_playlist_res(&data) {
                Ok(m3u8_rs::playlist::Playlist::MasterPlaylist(pl)) => {
                    println!("{:#?}", pl)
                }
                Ok(m3u8_rs::playlist::Playlist::MediaPlaylist(pl)) => {
                    println!("{:#?}", pl)
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }

        *res.status_mut() = StatusCode::NO_CONTENT;

        let header_value = HeaderValue::from_str(path.to_str().unwrap()).unwrap();

        res.headers_mut().insert(LOCATION, header_value);

        #[cfg(debug_assertions)]
        println!("{:#?}", res);

        return Ok(res);
    }

    let data = hyper::body::to_bytes(body).await?;

    #[cfg(debug_assertions)]
    println!("Bytes received => {}", data.len());

    let variant = path
        .parent()
        .expect("Orphan path!")
        .file_name()
        .expect("Dir with no name!")
        .to_os_string()
        .into_string()
        .expect("Dir name is invalid Unicode!");

    let msg = if path.extension().unwrap() == FMP4 {
        VideoData::Media(variant, data)
    } else if path.extension().unwrap() == MP4 {
        VideoData::Initialization(variant, data)
    } else {
        panic!("Not fmp4 or mp4");
    };

    if let Err(error) = collector.send(msg).await {
        eprintln!("Video receiver hung up! Error: {}", error);

        *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

        #[cfg(debug_assertions)]
        println!("{:#?}", res);

        return Ok(res);
    }

    *res.status_mut() = StatusCode::CREATED;

    let header_value = HeaderValue::from_str(path.to_str().unwrap()).unwrap();

    res.headers_mut().insert(LOCATION, header_value);

    #[cfg(debug_assertions)]
    println!("{:#?}", res);

    Ok(res)
}
