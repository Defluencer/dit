use crate::actors::{Archive, VideoData};
use crate::server::services::put_requests;

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::signal::ctrl_c;
use tokio::sync::mpsc::Sender;

use hyper::service::Service;
use hyper::{Body, Error, Request, Response, Server};

type FutureWrapper<T, U> = Pin<Box<dyn Future<Output = Result<T, U>> + Send>>;

struct IngessService {
    collector: Sender<VideoData>,
}

impl Service<Request<Body>> for IngessService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = FutureWrapper<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(put_requests(req, self.collector.clone()))
    }
}

struct MakeService {
    collector: Sender<VideoData>,
}

impl MakeService {
    fn new(collector: Sender<VideoData>) -> Self {
        Self { collector }
    }
}

impl<T> Service<T> for MakeService {
    type Response = IngessService;
    type Error = Error;
    type Future = FutureWrapper<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let collector = self.collector.clone();

        let fut = async move { Ok(IngessService { collector }) };

        Box::pin(fut)
    }
}

async fn shutdown_signal(mut archive_tx: Sender<Archive>) {
    ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");

    let msg = Archive::Finalize;

    if let Err(error) = archive_tx.send(msg).await {
        eprintln!("Archive receiver hung up {}", error);
    }
}

pub async fn start_server(
    server_addr: String,
    collector: Sender<VideoData>,
    archive_tx: Sender<Archive>,
) {
    let server_addr = server_addr
        .parse::<SocketAddr>()
        .expect("Parsing socket address failed");

    let service = MakeService::new(collector.clone());

    let server = Server::bind(&server_addr).serve(service);

    let graceful = server.with_graceful_shutdown(shutdown_signal(archive_tx));

    if let Err(e) = graceful.await {
        eprintln!("Server error {}", e);
    }
}
