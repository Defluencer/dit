use std::future::Future;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use hyper::service::Service;
use hyper::{Body, Error, Request, Response, Server};

use tokio::signal::ctrl_c;
use tokio::sync::RwLock;

use crate::playlist::Playlists;
use crate::services::{get_requests, PATH_MASTER};

type FutureWrapper<T, U> = Pin<Box<dyn Future<Output = Result<T, U>> + Send>>;

struct LiveLikeClientService {
    playlist: Arc<RwLock<Playlists>>,
}

impl Service<Request<Body>> for LiveLikeClientService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = FutureWrapper<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(get_requests(req, self.playlist.clone()))
    }
}

struct MakeLiveLikeClientService {
    playlist: Arc<RwLock<Playlists>>,
}

impl MakeLiveLikeClientService {
    fn new(playlist: Arc<RwLock<Playlists>>) -> Self {
        Self { playlist }
    }
}

impl<T> Service<T> for MakeLiveLikeClientService {
    type Response = LiveLikeClientService;
    type Error = Error;
    type Future = FutureWrapper<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let playlist = self.playlist.clone();

        let fut = async move { Ok(LiveLikeClientService { playlist }) };

        Box::pin(fut)
    }
}

async fn shutdown_signal() {
    ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

// Hard-Coded for now...
pub const SERVER_PORT: u16 = 2527;

pub async fn start_server(playlist: Arc<RwLock<Playlists>>) {
    let server_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), SERVER_PORT);

    let service = MakeLiveLikeClientService::new(playlist);

    let server = Server::bind(&server_addr).serve(service);

    println!("Watch live stream at http://{}{}", server_addr, PATH_MASTER);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("Server error {}", e);
    }
}
