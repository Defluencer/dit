use std::path::Path;

use hyper::body::Bytes;
use hyper::header::{HeaderValue, LOCATION};
use hyper::{Body, Error, Method, Request, Response, StatusCode};

use tokio::sync::mpsc::Sender;

pub async fn put_requests(
    req: Request<Body>,
    mut collector: Sender<(String, Bytes)>,
) -> Result<Response<Body>, Error> {
    #[cfg(debug_assertions)]
    println!("{:#?}", req);

    let mut res = Response::new(Body::empty());

    let (parts, body) = req.into_parts();

    if parts.method != Method::PUT {
        *res.status_mut() = StatusCode::NOT_FOUND;

        #[cfg(debug_assertions)]
        println!("{:#?}", res);

        return Ok(res);
    }

    let path = Path::new(parts.uri.path());

    //Ignore all except .ts video files
    if path.extension() == None || path.extension().unwrap() != "ts" {
        *res.status_mut() = StatusCode::NO_CONTENT;

        let header_value = HeaderValue::from_str(path.to_str().unwrap()).unwrap();

        res.headers_mut().insert(LOCATION, header_value);

        #[cfg(debug_assertions)]
        println!("{:#?}", res);

        return Ok(res);
    }

    let video_data = hyper::body::to_bytes(body).await?;

    #[cfg(debug_assertions)]
    println!("Bytes received => {}", video_data.len());

    let parent = path
        .parent()
        .expect("Orphan path!")
        .file_name()
        .expect("Dir with no name!")
        .to_os_string()
        .into_string()
        .expect("Dir name is invalid Unicode!");

    let msg = (parent, video_data);

    if let Err(error) = collector.send(msg).await {
        eprintln!("Collector hung up {}", error);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_parent() {
        let full_path = "/1080p60/356.ts";
        let path = Path::new(full_path);

        /* let file_name = "356";

        let file = path.file_stem().unwrap();

        assert_eq!(file, file_name); */

        let folder_name = "1080p60";

        let parent = path.parent().unwrap().file_name().unwrap();

        assert_eq!(parent, folder_name);
    }
}
