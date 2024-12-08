// #![allow(unused_imports, unused_variables)]
#![cfg_attr(
    debug_assertions,
    allow(unused_imports, unused_variables, dead_code, unused_mut)
)]
use hickory_client::client::{AsyncClient, Client, ClientConnection, ClientHandle, SyncClient};
use hickory_client::multicast::MdnsClientConnection;
use hickory_client::udp::UdpClientConnection;
use hickory_proto::multicast::MdnsClientStream;
use hickory_proto::udp::UdpClientStream;
use hickory_proto::xfer::BufDnsStreamHandle;
use log::{info, trace, warn};

use async_trait::async_trait;
// use futures_util::future::Lazy;
use futures_util::{pin_mut, stream::StreamExt};

use hickory_server::authority::{MessageRequest, MessageResponseBuilder};

use once_cell::sync::Lazy;

use std::pin::Pin;
use std::str::FromStr;

use std::time::Duration;

use hickory_proto::op::Header;
use hickory_server::proto::rr::{IntoName, LowerName, Name};
use hickory_server::server::{Request, ResponseInfo};
use hickory_server::server::{RequestHandler, ResponseHandler};

use tokio::net::TcpListener;
use tokio::net::UdpSocket;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use hickory_server::server::ServerFuture;

use network_interface::NetworkInterface;
use network_interface::NetworkInterfaceConfig;

struct MyResponseHandler {}

pub(crate) const MDNS_PORT: u16 = 5353;
/// mDNS ipv4 address, see [multicast-addresses](https://www.iana.org/assignments/multicast-addresses/multicast-addresses.xhtml)
pub static MDNS_IPV4: Lazy<SocketAddr> =
    Lazy::new(|| SocketAddr::new(Ipv4Addr::new(224, 0, 0, 251).into(), MDNS_PORT));
/// link-local mDNS ipv6 address, see [ipv6-multicast-addresses](https://www.iana.org/assignments/ipv6-multicast-addresses/ipv6-multicast-addresses.xhtml)
pub static MDNS_IPV6: Lazy<SocketAddr> = Lazy::new(|| {
    SocketAddr::new(
        Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x00FB).into(),
        MDNS_PORT,
    )
});
// const MDNS: Name = Name::from_labels(vec!["local"]).unwrap();
// const MDNS_TLD: Name = Name::from(LowerName::from_str("local").unwrap());

#[async_trait]
impl RequestHandler for MyResponseHandler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        info!("hit requesthandler");
        let question = request.request_info().query.name();
        info!("{question:?}");
        let local_zone = LowerName::from(Name::from_str("local").unwrap());
        // Check it's mDNS zone
        if !local_zone.zone_of(question) {
            panic!()
        }
        let hostname = question.into_name().unwrap().to_lowercase().to_string();
        info!("{hostname:?}");

        // let conn = MdnsClientConnection::new_ipv6(None, None);
        // let client = SyncClient::new(conn);
        // let aaaa_reply = client.query(
        //     &Name::from_str("mum.local.").unwrap(),
        //     hickory_proto::rr::DNSClass::IN,
        //     hickory_proto::rr::RecordType::AAAA,
        // );
        // info!("{aaaa_reply:?}");
        // let udp_stream = UdpClientStream::new(*MDNS_IPV6);
        let network_interfaces = NetworkInterface::show().unwrap();

        // Just manually dumping interfaces
        // for itf in network_interfaces.iter() {
        //     println!("{:?}", itf);
        // }
        let mdns_stream = MdnsClientStream::new_ipv6(
            hickory_proto::multicast::MdnsQueryType::OneShot,
            None,
            // Manually revealed interface index, must find auto detecting logic
            Some(2),
        );
        let (handle, _) = BufDnsStreamHandle::new(*MDNS_IPV6);
        let mut client = AsyncClient::new(mdns_stream.0, handle, None);
        let (mut foo, bar) = client.await.unwrap();
        foo.disable_edns();
        let aaaa_reply = foo
            .query(
                Name::from_str("mum.local.").unwrap(),
                hickory_proto::rr::DNSClass::IN,
                hickory_proto::rr::RecordType::AAAA,
            )
            .await;
        info!("{aaaa_reply:?}");

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
    info!("started");
    let request = server.block_until_done().await;
    let _request = request.unwrap();
}
