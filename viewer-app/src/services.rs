use crate::playlist::Playlist;
use hyper::{Body, Error, Method, Request, Response, StatusCode};
use std::sync::{Arc, RwLock};

const REQUEST_URI_PATH_MASTER: &str = "/live/master.m3u8";

const REQUEST_URI_PATH_1080P: &str = "/live/1080p/index.m3u8";
const REQUEST_URI_PATH_720P: &str = "/live/720p/index.m3u8";
const REQUEST_URI_PATH_480P: &str = "/live/480p/index.m3u8";

pub async fn get_requests(
    req: Request<Body>,
    data: Arc<RwLock<Playlist>>,
) -> Result<Response<Body>, Error> {
    let mut response = Response::new(Body::empty());

    if Method::GET != *req.method() {
        *response.status_mut() = StatusCode::NOT_FOUND;
        return Ok(response);
    }

    match req.uri().path() {
        //TODO output master playlist
        REQUEST_URI_PATH_MASTER => *response.body_mut() = Body::from("master playlist"),
        REQUEST_URI_PATH_1080P => get_playlist(&mut response, data),
        REQUEST_URI_PATH_720P => get_playlist(&mut response, data),
        REQUEST_URI_PATH_480P => get_playlist(&mut response, data),
        _ => *response.status_mut() = StatusCode::NOT_FOUND,
    }

    Ok(response)
}

fn get_playlist(response: &mut Response<Body>, data: Arc<RwLock<Playlist>>) {
    match data.read() {
        Ok(playlist) => *response.body_mut() = Body::from(playlist.to_str()),
        Err(e) => {
            eprintln!("Lock poisoned. {}", e);
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
}
