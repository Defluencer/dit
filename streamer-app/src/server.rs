use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::service::Service;
use hyper::{Body, Error, Request, Response, Server};
use ipfs_api::IpfsClient;
use tokio::signal::ctrl_c;

mod services;

use crate::services::put_requests;

type FutureWrapper<T, U> = Pin<Box<dyn Future<Output = Result<T, U>> + Send>>;

struct IPFSClientService {
    client: IpfsClient,
}

impl Service<Request<Body>> for IPFSClientService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = FutureWrapper<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(put_requests(req, self.client.clone()))
    }
}

struct MakeIPFSClientService {
    client: IpfsClient,
}

impl MakeIPFSClientService {
    fn new() -> Self {
        Self {
            client: IpfsClient::default(),
        }
    }
}

impl<T> Service<T> for MakeIPFSClientService {
    type Response = IPFSClientService;
    type Error = Error;
    type Future = FutureWrapper<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let client = self.client.clone();

        let fut = async move { Ok(IPFSClientService { client }) };

        Box::pin(fut)
    }
}

async fn shutdown_signal() {
    ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

const SERVER_PORT: u16 = 2424;

async fn start_server() {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), SERVER_PORT);

    let service = MakeIPFSClientService::new();

    let server = Server::bind(&server_addr).serve(service);

    println!("Listening on http://{}", server_addr);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("Server error: {}", e);
    }
}
