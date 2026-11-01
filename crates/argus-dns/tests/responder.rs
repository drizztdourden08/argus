//! The responder on real sockets: loopback with port 0, never port 53.
//! Keep rule: S2.
// test-tag: standard

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use argus_dns::{Responder, TYPE_A, TYPE_AAAA, Zone, ask, resolve};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

const ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 65, 82, 71);
const WAIT: Duration = Duration::from_secs(5);

struct Running {
    at: SocketAddr,
    tcp: bool,
    stop: CancellationToken,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}

async fn start() -> Running {
    let bind: SocketAddr = (Ipv4Addr::LOCALHOST, 0).into();
    let responder = Responder::bind(bind, Zone::new("argus", ADDRESS))
        .await
        .expect("binds");
    let at = responder.local_addr().expect("an address");
    let tcp = responder.has_tcp();
    let stop = CancellationToken::new();
    let task = tokio::spawn(responder.run(stop.clone()));
    Running {
        at,
        tcp,
        stop,
        task,
    }
}

impl Running {
    async fn end(self) {
        self.stop.cancel();
        self.task.await.expect("joins").expect("ends cleanly");
    }
}

#[tokio::test]
async fn udp_answers_a_aaaa_and_refuses_other_names() {
    let server = start().await;
    assert_eq!(
        resolve(server.at, "rotp.argus", WAIT)
            .await
            .expect("resolves"),
        [ADDRESS]
    );
    let aaaa = ask(server.at, "rotp.argus", TYPE_AAAA, WAIT)
        .await
        .expect("answers");
    assert_eq!((aaaa.rcode, aaaa.records), (0, 0));
    let other = ask(server.at, "example.com", TYPE_A, WAIT)
        .await
        .expect("answers");
    assert_eq!(other.rcode, 5);
    server.end().await;
}

#[tokio::test]
async fn tcp_answers_two_queries_on_one_connection() {
    let server = start().await;
    assert!(server.tcp, "the port UDP got was free for TCP");
    let mut stream = TcpStream::connect(server.at).await.expect("connects");
    for name in ["a.argus", "b.argus"] {
        let query = query_by_hand(name);
        stream
            .write_u16(u16::try_from(query.len()).expect("small"))
            .await
            .expect("len");
        stream.write_all(&query).await.expect("query");
        let len = stream.read_u16().await.expect("reply length");
        let mut reply = vec![0u8; usize::from(len)];
        stream.read_exact(&mut reply).await.expect("reply");
        assert_eq!(&reply[reply.len() - 4..], ADDRESS.octets(), "{name}");
    }
    server.end().await;
}

/// A query for `name`, type A, built by hand so the test does not lean on the crate's encoder.
fn query_by_hand(name: &str) -> Vec<u8> {
    let mut out = vec![0x12, 0x34, 0x01, 0x00, 0, 1, 0, 0, 0, 0, 0, 0];
    for label in name.split('.') {
        out.push(u8::try_from(label.len()).expect("short"));
        out.extend_from_slice(label.as_bytes());
    }
    out.extend_from_slice(&[0, 0, 1, 0, 1]);
    out
}
