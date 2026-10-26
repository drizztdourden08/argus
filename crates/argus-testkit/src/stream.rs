//! An SSE reader: frames parsed as they arrive, comments kept apart, the raw text kept for byte
//! checks, and `next` waiting for an event by name or predicate (`None` on timeout or end).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::Request;
use crate::client::{dechunk, parse_head};

/// One event.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// The `id` line.
    pub id: Option<String>,
    /// The `event` line (`message` when absent).
    pub event: String,
    /// The `data` line as JSON.
    pub data: Value,
}

#[derive(Default)]
struct Inner {
    events: VecDeque<Event>,
    comments: Vec<String>,
    raw: String,
    pending: String,
    chunks: Vec<u8>,
    ended: bool,
}

impl Inner {
    fn take_chunks(&mut self) {
        loop {
            let Some(at) = self.chunks.windows(2).position(|w| w == b"\r\n") else {
                return;
            };
            let size =
                usize::from_str_radix(String::from_utf8_lossy(&self.chunks[..at]).trim(), 16)
                    .unwrap_or(0);
            if self.chunks.len() < at + 4 + size {
                return;
            }
            let text =
                String::from_utf8_lossy(&dechunk(&self.chunks[..at + 4 + size])).into_owned();
            self.chunks.drain(..at + 4 + size);
            self.raw.push_str(&text);
            self.pending.push_str(&text);
        }
    }

    fn take_frames(&mut self) {
        while let Some(at) = self.pending.find("\n\n") {
            let block: String = self.pending.drain(..at + 2).collect();
            self.frame(block.trim_end_matches('\n'));
        }
    }

    fn frame(&mut self, block: &str) {
        let mut ev = Event {
            id: None,
            event: "message".into(),
            data: Value::Null,
        };
        let mut has_data = false;
        for line in block.split('\n') {
            if let Some(c) = line.strip_prefix(':') {
                self.comments.push(c.trim().to_owned());
            } else if let Some(v) = line.strip_prefix("id: ") {
                ev.id = Some(v.to_owned());
            } else if let Some(v) = line.strip_prefix("event: ") {
                v.clone_into(&mut ev.event);
            } else if let Some(v) = line.strip_prefix("data: ") {
                ev.data = serde_json::from_str(v).unwrap_or(Value::Null);
                has_data = true;
            }
        }
        if has_data {
            self.events.push_back(ev);
        }
    }
}

/// A doc, wake or log stream being read.
pub struct SseReader {
    /// The answer status.
    pub status: u16,
    /// The answer headers.
    pub headers: Vec<(String, String)>,
    /// The error body when the status is not 200.
    pub body: Value,
    inner: Arc<Mutex<Inner>>,
    wake: Arc<Notify>,
    task: Option<JoinHandle<()>>,
}

async fn pump(mut s: TcpStream, rest: Vec<u8>, inner: Arc<Mutex<Inner>>, wake: Arc<Notify>) {
    let feed = |bytes: &[u8]| {
        let mut st = inner.lock().unwrap_or_else(PoisonError::into_inner);
        st.chunks.extend_from_slice(bytes);
        st.take_chunks();
        st.take_frames();
    };
    feed(&rest);
    wake.notify_waiters();
    let mut buf = vec![0u8; 16 * 1024];
    while let Ok(n) = s.read(&mut buf).await {
        if n == 0 {
            break;
        }
        feed(&buf[..n]);
        wake.notify_waiters();
    }
    inner.lock().unwrap_or_else(PoisonError::into_inner).ended = true;
    wake.notify_waiters();
}

async fn read_head(s: &mut TcpStream) -> (String, Vec<u8>) {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        if let Some(at) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..at]).into_owned();
            return (head, buf[at + 4..].to_vec());
        }
        match s.read(&mut chunk).await {
            Ok(n) if n > 0 => buf.extend_from_slice(&chunk[..n]),
            _ => return (String::from_utf8_lossy(&buf).into_owned(), Vec::new()),
        }
    }
}

const OPEN_LIMIT: Duration = Duration::from_secs(10);

/// The body of a plain answer on a kept-alive connection: `Content-Length` bytes.
async fn read_body(s: &mut TcpStream, mut body: Vec<u8>, headers: &[(String, String)]) -> Vec<u8> {
    let want = headers
        .iter()
        .find(|(k, _)| k == "content-length")
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut chunk = [0u8; 4096];
    while body.len() < want {
        match s.read(&mut chunk).await {
            Ok(n) if n > 0 => body.extend_from_slice(&chunk[..n]),
            _ => break,
        }
    }
    body
}

impl SseReader {
    /// Opens a stream.
    ///
    /// # Panics
    /// When the server cannot be reached or does not answer within 10 s.
    pub async fn open(port: u16, req: &Request) -> SseReader {
        tokio::time::timeout(OPEN_LIMIT, Self::connect(port, req))
            .await
            .expect("the stream answers within 10 s")
    }

    async fn connect(port: u16, req: &Request) -> SseReader {
        let mut s = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect to the test server");
        s.write_all(req.head(port, false).as_bytes())
            .await
            .expect("send head");
        let (head, rest) = read_head(&mut s).await;
        let (status, headers) = parse_head(&head);
        let inner = Arc::new(Mutex::new(Inner::default()));
        let wake = Arc::new(Notify::new());
        if status != 200 {
            let body = read_body(&mut s, rest, &headers).await;
            let body = serde_json::from_slice(&body).unwrap_or(Value::Null);
            inner.lock().unwrap_or_else(PoisonError::into_inner).ended = true;
            return SseReader {
                status,
                headers,
                body,
                inner,
                wake,
                task: None,
            };
        }
        let task = tokio::spawn(pump(s, rest, Arc::clone(&inner), Arc::clone(&wake)));
        SseReader {
            status,
            headers,
            body: Value::Null,
            inner,
            wake,
            task: Some(task),
        }
    }

    fn take(&self, f: &dyn Fn(&Event) -> bool) -> Result<Option<Event>, ()> {
        let mut st = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(i) = st.events.iter().position(f) {
            return Ok(st.events.remove(i));
        }
        if st.ended { Err(()) } else { Ok(None) }
    }

    /// The next event accepted by `f`, removed from the queue.
    pub async fn next_where(&self, f: impl Fn(&Event) -> bool, ms: u64) -> Option<Event> {
        let until = tokio::time::Instant::now() + Duration::from_millis(ms);
        loop {
            let notified = self.wake.notified();
            match self.take(&f) {
                Ok(Some(e)) => return Some(e),
                Err(()) => return None,
                Ok(None) => {}
            }
            if tokio::time::timeout_at(until, notified).await.is_err() {
                return None;
            }
        }
    }

    /// The next event named `name` (any event when `None`).
    pub async fn next(&self, name: Option<&str>, ms: u64) -> Option<Event> {
        self.next_where(|e| name.is_none_or(|n| e.event == n), ms)
            .await
    }

    /// Events read and not taken yet.
    #[must_use]
    pub fn queued(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .events
            .len()
    }

    /// The comments seen (`ping`).
    #[must_use]
    pub fn comments(&self) -> Vec<String> {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .comments
            .clone()
    }

    /// Everything read, as text.
    #[must_use]
    pub fn raw(&self) -> String {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .raw
            .clone()
    }

    /// The server ended the stream.
    #[must_use]
    pub fn ended(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .ended
    }

    /// Drops the connection.
    pub fn close(mut self) {
        if let Some(t) = self.task.take() {
            t.abort();
        }
    }
}

impl Drop for SseReader {
    fn drop(&mut self) {
        if let Some(t) = self.task.take() {
            t.abort();
        }
    }
}
