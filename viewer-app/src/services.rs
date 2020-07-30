use std::sync::Arc;

use tokio::sync::RwLock;

use hyper::{Body, Error, Method, Request, Response, StatusCode};

use crate::playlist::Playlists;

// Hard-Coded for now...
pub const PATH_MASTER: &str = "/livelike/master.m3u8";
pub const PATH_1080_60: &str = "/livelike/1080p60/index.m3u8";
pub const PATH_720_60: &str = "/livelike/720p60/index.m3u8";
pub const PATH_720_30: &str = "/livelike/720p30/index.m3u8";
pub const PATH_480_30: &str = "/livelike/480p30/index.m3u8";

pub async fn get_requests(
    req: Request<Body>,
    data: Arc<RwLock<Playlists>>,
) -> Result<Response<Body>, Error> {
    let mut response = Response::new(Body::empty());

    if Method::GET != *req.method() {
        *response.status_mut() = StatusCode::NOT_FOUND;
        return Ok(response);
    }

    let mut buf: Vec<u8> = Vec::new();

    {
        let playlists = data.read().await;

        match req.uri().path() {
            PATH_MASTER => playlists
                .master
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_1080_60 => playlists
                .playlist_1080_60
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_720_60 => playlists
                .playlist_720_60
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_720_30 => playlists
                .playlist_720_30
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            PATH_480_30 => playlists
                .playlist_480_30
                .write_to(&mut buf)
                .expect("Can't write to buffer"),
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
                return Ok(response);
            }
        };
    }

    let string = String::from_utf8(buf).expect("Invalid UTF-8");

    *response.body_mut() = Body::from(string);

    Ok(response)
}
