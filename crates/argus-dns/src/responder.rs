//! The responder on its sockets: UDP, and TCP on the same port when it can be bound. Each query
//! is answered from the [`Zone`] at once; nothing here leaves the socket it was bound to.

use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::zone::Zone;

/// The largest query read from a socket.
const QUERY_MAX: usize = 4096;
/// How long a TCP client may stay silent.
const TCP_IDLE: Duration = Duration::from_secs(5);

/// A bound responder; [`Responder::run`] serves it.
pub struct Responder {
    zone: Arc<Zone>,
    udp: UdpSocket,
    tcp: Option<TcpListener>,
}

impl Responder {
    /// Binds UDP on `addr`, then TCP on the port UDP got. A TCP port that cannot be bound is not
    /// an error: the responder then answers UDP only.
    ///
    /// # Errors
    ///
    /// When the UDP socket cannot be bound.
    pub async fn bind(addr: SocketAddr, zone: Zone) -> io::Result<Self> {
        let udp = UdpSocket::bind(addr).await?;
        let tcp_addr = SocketAddr::new(addr.ip(), udp.local_addr()?.port());
        let tcp = TcpListener::bind(tcp_addr).await.ok();
        Ok(Self {
            zone: Arc::new(zone),
            udp,
            tcp,
        })
    }

    /// The address of the UDP socket.
    ///
    /// # Errors
    ///
    /// When the socket cannot tell.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.udp.local_addr()
    }

    /// Whether TCP is served too.
    #[must_use]
    pub fn has_tcp(&self) -> bool {
        self.tcp.is_some()
    }

    /// Answers queries until `stop` rings.
    ///
    /// # Errors
    ///
    /// When a socket fails for good.
    pub async fn run(self, stop: CancellationToken) -> io::Result<()> {
        let Self { zone, udp, tcp } = self;
        let over_tcp = serve_tcp(tcp, Arc::clone(&zone));
        tokio::select! {
            done = serve_udp(&udp, &zone) => done,
            () = over_tcp => Ok(()),
            () = stop.cancelled() => Ok(()),
        }
    }
}

/// A client that went away makes Windows fail the next receive; that is not the socket's end.
fn is_transient(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted | ErrorKind::Interrupted
    )
}

async fn serve_udp(udp: &UdpSocket, zone: &Zone) -> io::Result<()> {
    let mut buf = vec![0u8; QUERY_MAX];
    loop {
        let (len, from) = match udp.recv_from(&mut buf).await {
            Ok(got) => got,
            Err(error) if is_transient(&error) => continue,
            Err(error) => return Err(error),
        };
        if let Some(reply) = zone.answer(&buf[..len]) {
            // A client that is gone is not our trouble.
            let _gone = udp.send_to(&reply, from).await;
        }
    }
}

async fn serve_tcp(listener: Option<TcpListener>, zone: Arc<Zone>) {
    let Some(listener) = listener else {
        return std::future::pending().await;
    };
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(talk(stream, Arc::clone(&zone)));
            }
            Err(error) if is_transient(&error) => {}
            Err(_) => return,
        }
    }
}

/// Reads one length-prefixed query at a time and answers it, until the client is silent for
/// [`TCP_IDLE`], closes, or sends nonsense.
async fn talk(mut stream: TcpStream, zone: Arc<Zone>) {
    while let Ok(Ok(query)) = timeout(TCP_IDLE, read_message(&mut stream)).await {
        let Some(reply) = zone.answer(&query) else {
            return;
        };
        if write_message(&mut stream, &reply).await.is_err() {
            return;
        }
    }
}

async fn read_message(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let len = usize::from(stream.read_u16().await?);
    let mut message = vec![0u8; len];
    stream.read_exact(&mut message).await?;
    Ok(message)
}

async fn write_message(stream: &mut TcpStream, message: &[u8]) -> io::Result<()> {
    let len = u16::try_from(message.len()).map_err(|_| ErrorKind::InvalidData)?;
    stream.write_u16(len).await?;
    stream.write_all(message).await
}
