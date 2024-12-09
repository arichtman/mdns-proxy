// #![allow(unused_imports, unused_variables)]
#![cfg_attr(
    debug_assertions,
    allow(unused_imports, unused_variables, dead_code, unused_mut)
)]
use hickory_client::client::{AsyncClient, Client, ClientConnection, ClientHandle, SyncClient};
use hickory_client::multicast::MdnsClientConnection;
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_client::udp::UdpClientConnection;

use hickory_client::tcp::TcpClientStream;
use hickory_proto::multicast::{MdnsClientStream, MDNS_IPV4, MDNS_IPV6};
use hickory_proto::udp::UdpClientStream;

use hickory_proto::xfer::BufDnsStreamHandle;
use log::{debug, info, trace, warn};

use async_trait::async_trait;
use futures_util::{pin_mut, stream::StreamExt};

use hickory_server::authority::{MessageRequest, MessageResponseBuilder};

use once_cell::sync::Lazy;

use core::panic;
use std::pin::Pin;
use std::str::FromStr;

use std::time::Duration;

use hickory_proto::op::Header;
use hickory_server::proto::rr::{IntoName, LowerName, Name};
use hickory_server::server::{Request, ResponseInfo};
use hickory_server::server::{RequestHandler, ResponseHandler};

use tokio::net::TcpListener;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::net::UdpSocket;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use hickory_server::server::ServerFuture;

use network_interface::NetworkInterface;
use network_interface::NetworkInterfaceConfig;

struct MyResponseHandler {}

const MDNS_LOWERNAME: Lazy<LowerName> =
    Lazy::new(|| LowerName::from(Name::from_str("local").unwrap()));

#[async_trait]
impl RequestHandler for MyResponseHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        debug!("hit requesthandler");
        let question = request.request_info().query.name();
        dbg!(question);
        // Check it's mDNS zone
        if !MDNS_LOWERNAME.zone_of(question) {
            panic!()
        }
        let hostname = question.into_name().unwrap().to_lowercase().to_string();
        dbg!(hostname);

        let network_interfaces = NetworkInterface::show().unwrap();

        // Just manually dumping interfaces
        // for itf in network_interfaces.iter() {
        //     println!("{:?}", itf);
        // }

        let (mdns_stream, mdns_sender) = MdnsClientStream::new_ipv6(
            hickory_proto::multicast::MdnsQueryType::OneShot,
            None,
            // Manually revealed interface index, must find auto detecting logic
            Some(2),
        );
        let client = AsyncClient::new(mdns_stream, mdns_sender, None);
        let (mut client, bg) = client.await.unwrap();
        client.disable_edns();
        tokio::spawn(bg);
        let query = client.query(
            Name::from_str("mum.local.").unwrap(),
            hickory_proto::rr::DNSClass::IN,
            hickory_proto::rr::RecordType::AAAA,
        );
        let response = query.await.unwrap();
        dbg!(response);
        ResponseInfo::from(Header::default())
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let handler = MyResponseHandler {};
    let mut server = ServerFuture::new(handler);
    let sock = UdpSocket::bind("[::1]:1053").await.unwrap();
    let tcp_listener = TcpListener::bind("[::1]:1053").await.unwrap();
    server.register_socket(sock);
    server.register_listener(tcp_listener, Duration::from_secs(2));
    info!("Started");
    let request = server.block_until_done().await;
    let _request = request.unwrap();
}
