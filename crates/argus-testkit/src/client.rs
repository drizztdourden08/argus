//! A small HTTP/1.1 client: one request per connection, any header (`Host` included), the whole
//! answer read back. Enough to drive the server the way the Node tests do with `node:http`.

use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// A request.
#[derive(Debug, Clone, Default)]
pub struct Request {
    /// The method.
    pub method: String,
    /// The path with its query.
    pub path: String,
    /// Extra headers.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
    /// A pause between the head and the body, as a slow writer makes.
    pub gap: Duration,
}

impl Request {
    /// A request with no body.
    #[must_use]
    pub fn new(method: &str, path: &str) -> Self {
        Request {
            method: method.to_owned(),
            path: path.to_owned(),
            ..Request::default()
        }
    }

    /// The same request with one more header.
    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// The same request with a raw body.
    #[must_use]
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// The same request, its body sent `gap` after its head.
    #[must_use]
    pub fn paced(mut self, gap: Duration) -> Self {
        self.gap = gap;
        self
    }

    /// The same request with a JSON body and `Content-Type: application/json` unless one is set.
    #[must_use]
    pub fn json(self, v: &Value) -> Self {
        let typed = self.has("content-type");
        let r = self.body(v.to_string());
        if typed {
            r
        } else {
            r.header("Content-Type", "application/json")
        }
    }

    /// Whether a header is set.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case(name))
    }

    pub(crate) fn head(&self, port: u16, close: bool) -> String {
        let mut lines = vec![format!("{} {} HTTP/1.1", self.method, self.path)];
        if !self.has("host") {
            lines.push(format!("Host: 127.0.0.1:{port}"));
        }
        lines.extend(self.headers.iter().map(|(k, v)| format!("{k}: {v}")));
        if !self.body.is_empty() || !["GET", "HEAD"].contains(&self.method.as_str()) {
            lines.push(format!("Content-Length: {}", self.body.len()));
        }
        let mut out = lines.join("\r\n") + "\r\n";
        if close {
            out.push_str("Connection: close\r\n");
        }
        out.push_str("\r\n");
        out
    }
}

/// An answer.
#[derive(Debug, Clone)]
pub struct Reply {
    /// The status.
    pub status: u16,
    /// The headers, names lowercase.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
}

impl Reply {
    /// The first header of that name.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// The body as text.
    #[must_use]
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// The body as JSON (`null` when it is not JSON).
    #[must_use]
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or(Value::Null)
    }

    /// The error code of an error answer.
    #[must_use]
    pub fn code(&self) -> String {
        self.json()["error"]["code"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    }
}

pub(crate) fn parse_head(head: &str) -> (u16, Vec<(String, String)>) {
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|l| l.split(' ').nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_owned()))
        .collect();
    (status, headers)
}

pub(crate) fn dechunk(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = raw;
    while let Some(at) = rest.windows(2).position(|w| w == b"\r\n") {
        let size =
            usize::from_str_radix(String::from_utf8_lossy(&rest[..at]).trim(), 16).unwrap_or(0);
        if size == 0 || rest.len() < at + 2 + size {
            break;
        }
        out.extend_from_slice(&rest[at + 2..at + 2 + size]);
        rest = rest.get(at + 4 + size..).unwrap_or_default();
    }
    out
}

/// Sends a request to `127.0.0.1:<port>` and reads the whole answer (status 0 when none came).
///
/// # Panics
/// When the server cannot be reached (a test failure).
pub async fn request(port: u16, req: &Request) -> Reply {
    tokio::time::timeout(std::time::Duration::from_secs(30), exchange(port, req))
        .await
        .expect("the server answers within 30 s")
}

async fn exchange(port: u16, req: &Request) -> Reply {
    let mut s = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect to the test server");
    s.write_all(req.head(port, true).as_bytes())
        .await
        .expect("send head");
    if !req.body.is_empty() {
        tokio::time::sleep(req.gap).await;
    }
    let _ = s.write_all(&req.body).await;
    let mut buf = Vec::new();
    let _ = s.read_to_end(&mut buf).await;
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap_or(buf.len());
    let (status, headers) = parse_head(&String::from_utf8_lossy(&buf[..split]));
    let raw = buf.get(split + 4..).unwrap_or_default();
    let chunked = headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v.contains("chunked"));
    Reply {
        status,
        headers,
        body: if chunked { dechunk(raw) } else { raw.to_vec() },
    }
}
