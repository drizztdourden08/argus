//! Asks a responder one question over UDP: what `argus dns status` and the tests do.

use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::wire::{Answer, TYPE_A, encode_query, parse_answer};

const QUERY_ID: u16 = 0x4152;
const ANSWER_MAX: usize = 4096;

/// Sends one query for `name` and `qtype` to `server` and reads the reply.
///
/// # Errors
///
/// When the socket fails, nothing answers within `wait`, or the reply is not a DNS message of
/// this query.
pub async fn ask(server: SocketAddr, name: &str, qtype: u16, wait: Duration) -> io::Result<Answer> {
    let bind: SocketAddr = (Ipv4Addr::LOCALHOST, 0).into();
    let socket = UdpSocket::bind(bind).await?;
    socket.connect(server).await?;
    socket.send(&encode_query(QUERY_ID, name, qtype)).await?;
    let mut buf = vec![0u8; ANSWER_MAX];
    let len = timeout(wait, socket.recv(&mut buf))
        .await
        .map_err(|_| io::Error::new(ErrorKind::TimedOut, "no answer"))??;
    parse_answer(&buf[..len])
        .filter(|answer| answer.id == QUERY_ID)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "not an answer to the query"))
}

/// The addresses `server` gives for `name`; empty when it answers with none.
///
/// # Errors
///
/// As [`ask`], and when the answer is an error code.
pub async fn resolve(server: SocketAddr, name: &str, wait: Duration) -> io::Result<Vec<Ipv4Addr>> {
    let answer = ask(server, name, TYPE_A, wait).await?;
    if answer.rcode == 0 {
        Ok(answer.addresses)
    } else {
        Err(io::Error::other(format!(
            "the server answered code {}",
            answer.rcode
        )))
    }
}
