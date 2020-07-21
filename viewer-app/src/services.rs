use crate::playlist::Playlists;
use hyper::{Body, Error, Method, Request, Response, StatusCode};
use std::sync::{Arc, RwLock};

const REQUEST_URI_PATH_MASTER: &str = "/live/master.m3u8";

pub const REQUEST_URI_PATH_1080_60: &str = "/live/1080_60/index.m3u8";
pub const REQUEST_URI_PATH_720_60: &str = "/live/720_60/index.m3u8";
pub const REQUEST_URI_PATH_720_30: &str = "/live/720_30/index.m3u8";
pub const REQUEST_URI_PATH_480_30: &str = "/live/480_30/index.m3u8";

pub async fn get_requests(
    req: Request<Body>,
    data: Arc<RwLock<Playlists>>,
) -> Result<Response<Body>, Error> {
    let mut response = Response::new(Body::empty());

    if Method::GET != *req.method() {
        *response.status_mut() = StatusCode::NOT_FOUND;
        return Ok(response);
    }

    let playlists = data.read().expect("Lock Poisoned");

    let mut buf: Vec<u8> = Vec::new();

    match req.uri().path() {
        REQUEST_URI_PATH_MASTER => playlists
            .master
            .write_to(&mut buf)
            .expect("Can't write to buffer"),
        REQUEST_URI_PATH_1080_60 => playlists
            .playlist_1080_60
            .write_to(&mut buf)
            .expect("Can't write to buffer"),
        REQUEST_URI_PATH_720_60 => playlists
            .playlist_720_60
            .write_to(&mut buf)
            .expect("Can't write to buffer"),
        REQUEST_URI_PATH_720_30 => playlists
            .playlist_720_30
            .write_to(&mut buf)
            .expect("Can't write to buffer"),
        REQUEST_URI_PATH_480_30 => playlists
            .playlist_480_30
            .write_to(&mut buf)
            .expect("Can't write to buffer"),
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response);
        }
    };

    *response.body_mut() = Body::from(buf);

    Ok(response)
}
