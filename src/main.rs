// #![allow(unused_imports, unused_variables)]
#![cfg_attr(
    debug_assertions,
    allow(unused_imports, unused_variables, dead_code, unused_mut)
)]
use async_trait::async_trait;
use hickory_server::authority::{MessageRequest, MessageResponseBuilder};
use mdns::Response;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use hickory_proto::op::Header;
use hickory_server::proto::rr::{IntoName, LowerName, Name};
use hickory_server::server::{Request, ResponseInfo};
use hickory_server::server::{RequestHandler, ResponseHandler};

use tokio::net::TcpListener;
use tokio::net::UdpSocket;

use hickory_server::server::ServerFuture;

struct MyResponseHandler {}

// const MDNS: Name = Name::from_labels(vec!["local"]).unwrap();
// const MDNS_TLD: Name = Name::from(LowerName::from_str("local").unwrap());

#[async_trait]
impl RequestHandler for MyResponseHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        let question = request.request_info().query.name();
        if !question.zone_of(&LowerName::from_str("local").unwrap()) {
            panic!()
        }
        let hostname = question.into_name().unwrap().to_lowercase().to_string();
        let response = mdns::resolve::one(hostname.as_str(), &hostname, Duration::from_secs(5))
            .await
            .unwrap();
        dbg!(response);
        ResponseInfo::from(Header::default())
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
