use std::path::Path;

use hyper::body::Bytes;
use hyper::header::{HeaderValue, CONTENT_LOCATION};
use hyper::{Body, Error, Method, Request, Response, StatusCode};

use tokio::sync::mpsc::Sender;

use crate::collector::StreamVariants;

// Hard-Coded for now...
const PATH_1080_60: &str = "/livelike/1080p60";
const PATH_720_60: &str = "/livelike/720p60";
const PATH_720_30: &str = "/livelike/720p30";
const PATH_480_30: &str = "/livelike/480p30";

pub async fn put_requests(
    req: Request<Body>,
    mut collector: Sender<(StreamVariants, Bytes)>,
) -> Result<Response<Body>, Error> {
    #[cfg(debug_assertions)]
    println!("{:#?}", req);

    let mut response = Response::new(Body::empty());

    let (parts, body) = req.into_parts();

    if parts.method != Method::PUT {
        *response.status_mut() = StatusCode::NOT_FOUND;
        return Ok(response);
    }

    let path = Path::new(parts.uri.path());

    //Ignore all except .ts video files
    if path.extension() == None || path.extension().unwrap() != "ts" {
        *response.status_mut() = StatusCode::NO_CONTENT;

        let header_value = HeaderValue::from_str(path.to_str().unwrap()).unwrap();

        response
            .headers_mut()
            .insert(CONTENT_LOCATION, header_value);

        return Ok(response);
    }

    let video_data = hyper::body::to_bytes(body).await?;

    #[cfg(debug_assertions)]
    println!("Bytes received => {}", video_data.len());

    let parent = path
        .parent()
        .expect("Orphan path")
        .to_str()
        .expect("Path invalid UTF-8");

    let variant: StreamVariants;

    match parent {
        PATH_1080_60 => variant = StreamVariants::Stream1080p60,
        PATH_720_60 => variant = StreamVariants::Stream720p60,
        PATH_720_30 => variant = StreamVariants::Stream720p30,
        PATH_480_30 => variant = StreamVariants::Stream480p30,
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response);
        }
    };

    let msg = (variant, video_data);

    if let Err(error) = collector.send(msg).await {
        eprintln!("Collector hung up {}", error);
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(response);
    }

    *response.status_mut() = StatusCode::CREATED;

    let header_value = HeaderValue::from_str(path.to_str().unwrap()).unwrap();

    response
        .headers_mut()
        .insert(CONTENT_LOCATION, header_value);

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_parent() {
        let full_path = "/livelike/1080p60/0.ts";
        let path = Path::new(full_path);

        let folder = "/livelike/1080p60";

        let parent = path.parent().unwrap().to_str().unwrap();

        assert_eq!(parent, folder);
    }
}
