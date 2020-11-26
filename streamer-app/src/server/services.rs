use std::path::Path;

use tokio::sync::mpsc::Sender;

use hyper::body::Bytes;
use hyper::header::{HeaderValue, LOCATION};
use hyper::{Body, Error, Method, Request, Response, StatusCode};

pub async fn put_requests(
    req: Request<Body>,
    mut collector: Sender<(String, Bytes, bool)>,
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

    //Ignore .m3u8 files
    if path.extension() == None || path.extension().unwrap() == "m3u8" {
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

    let init = path.extension().unwrap() == "mp4";

    let variant = path
        .parent()
        .expect("Orphan path!")
        .file_name()
        .expect("Dir with no name!")
        .to_os_string()
        .into_string()
        .expect("Dir name is invalid Unicode!");

    let msg = (variant, video_data, init);

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
