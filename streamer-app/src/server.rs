use std::future::Future;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::body::Bytes;
use hyper::service::Service;
use hyper::{Body, Error, Request, Response, Server};

use tokio::signal::ctrl_c;
use tokio::sync::mpsc::Sender;

use crate::collector::StreamVariants;
use crate::services::put_requests;

type FutureWrapper<T, U> = Pin<Box<dyn Future<Output = Result<T, U>> + Send>>;

struct LiveLikeService {
    collector: Sender<(StreamVariants, Bytes)>,
}

impl Service<Request<Body>> for LiveLikeService {
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

struct MakeLiveLikeService {
    collector: Sender<(StreamVariants, Bytes)>,
}

impl MakeLiveLikeService {
    fn new(collector: Sender<(StreamVariants, Bytes)>) -> Self {
        Self { collector }
    }
}

impl<T> Service<T> for MakeLiveLikeService {
    type Response = LiveLikeService;
    type Error = Error;
    type Future = FutureWrapper<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let collector = self.collector.clone();

        let fut = async move { Ok(LiveLikeService { collector }) };

        Box::pin(fut)
    }
}

async fn shutdown_signal() {
    ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");

    //TODO stamp the last dag node, finalizing the stream.
}

// Hard-Coded for now...
pub const SERVER_PORT: u16 = 2526;

pub async fn start_server(collector: Sender<(StreamVariants, Bytes)>) {
    let server_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), SERVER_PORT);

    let service = MakeLiveLikeService::new(collector);

    let server = Server::bind(&server_addr).serve(service);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("Server error {}", e);
    }
}
