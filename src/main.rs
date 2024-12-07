// #![allow(unused_imports, unused_variables)]
#![cfg_attr(debug_assertions, allow(unused_imports, unused_variables))]
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use hickory_server::proto::rr::{IntoName, LowerName, Name};
use hickory_server::server::RequestHandler;
use hickory_server::server::{Request, ResponseInfo};

use tokio::net::TcpListener;
use tokio::net::UdpSocket;

use hickory_server::server::ServerFuture;

struct MyResponseHandler {}

// const MDNS: Name = Name::from_labels(vec!["local"]).unwrap();
// const MDNS_TLD: Name = Name::from(LowerName::from_str("local").unwrap());

impl RequestHandler for MyResponseHandler {
    async fn handle_request<'life0, 'life1, 'async_trait, R>(
        &'life0 self,
        request: &'life1 Request,
        response_handle: R,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = hickory_server::server::ResponseInfo>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        R: 'async_trait + hickory_server::server::ResponseHandler,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        let question = request.request_info().query.name();
        if !question.zone_of(&LowerName::from_str("local").unwrap()) {
            panic!()
        }
        let hostname = question.into_name().unwrap().to_lowercase().to_string();
        let response = mdns::resolve::one(hostname.as_str(), &hostname, Duration::from_secs(5));
        // Pin::new(Box::new(response))
        Box::pin(response)
    }
}
#[tokio::main]
async fn main() {
    let handler = hickory_server::authority::Catalog::default();
    let mut server = ServerFuture::new(handler);
    let sock = UdpSocket::bind("[::1]:1053").await.unwrap();
    let tcp_listener = TcpListener::bind("[::1]:1053").await.unwrap();
    server.register_socket(sock);
    server.register_listener(tcp_listener, Duration::from_secs(5));
    loop {
        let request = server.block_until_done().await;
        let _request = request.unwrap();
    }
}
