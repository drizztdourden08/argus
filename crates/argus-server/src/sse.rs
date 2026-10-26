//! Server-sent events written by hand, so the bytes match the Node server: `id`, `event`, `data`
//! lines, a blank line after each frame, the data as one JSON line, and a `: ping` comment on the
//! heartbeat. A client more than a megabyte behind is dropped; it reconnects and resumes.

use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

use axum::body::Body;
use axum::http::header;
use axum::response::Response;
use bytes::Bytes;
use futures_core::Stream;
use serde_json::Value;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::{Instant, Interval, interval_at};

use crate::reply::bytes_reply;

mod fanout;
mod run_logs;

pub use fanout::{Fanout, MAX_CLIENTS_PER_SCOPE, StreamRequest};
pub use run_logs::{LogListening, RunLogs};

/// A client this far behind is dropped.
pub const SSE_BUFFER_LIMIT: usize = 1024 * 1024;

/// One frame: `id: <id>\nevent: <name>\ndata: <json>\n\n`.
#[must_use]
pub fn frame(name: &str, data: &Value, id: Option<i64>) -> String {
    frame_with(name, data, id.map(|id| id.to_string()).as_deref())
}

/// A frame whose id is text, such as the wake ids `W-12`.
#[must_use]
pub fn frame_with(name: &str, data: &Value, id: Option<&str>) -> String {
    let head = id.map(|id| format!("id: {id}\n")).unwrap_or_default();
    let data = argus_core::json_text::to_string(data).unwrap_or_default();
    format!("{head}event: {name}\ndata: {data}\n\n")
}

/// The writing side of one stream.
pub struct SseSender {
    tx: UnboundedSender<Bytes>,
    queued: Arc<AtomicUsize>,
    closed: Arc<AtomicBool>,
}

impl SseSender {
    fn push(&self, text: String) -> bool {
        if self.closed.load(Ordering::SeqCst) {
            return false;
        }
        let len = text.len();
        if self.queued.fetch_add(len, Ordering::SeqCst) + len > SSE_BUFFER_LIMIT
            || self.tx.send(Bytes::from(text)).is_err()
        {
            self.close();
            return false;
        }
        true
    }

    /// Writes an event; `false` once the stream is closed or the client fell behind.
    #[must_use]
    pub fn event(&self, name: &str, data: &Value, id: Option<i64>) -> bool {
        self.push(frame(name, data, id))
    }

    /// Writes an event with a text id; `false` once the stream is closed or the client fell
    /// behind.
    #[must_use]
    pub fn event_with(&self, name: &str, data: &Value, id: &str) -> bool {
        self.push(frame_with(name, data, Some(id)))
    }

    /// Ends the stream after what is queued.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

type OnDrop = Box<dyn FnOnce() + Send>;

struct SseBody {
    rx: UnboundedReceiver<Bytes>,
    queued: Arc<AtomicUsize>,
    closed: Arc<AtomicBool>,
    beat: Interval,
    open: Arc<AtomicUsize>,
    on_drop: Option<OnDrop>,
}

impl Stream for SseBody {
    type Item = Result<Bytes, Infallible>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.rx.poll_recv(cx) {
            Poll::Ready(Some(b)) => {
                this.queued.fetch_sub(b.len(), Ordering::SeqCst);
                return Poll::Ready(Some(Ok(b)));
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => {}
        }
        if this.closed.load(Ordering::SeqCst) {
            return Poll::Ready(None);
        }
        if this.beat.poll_tick(cx).is_ready() {
            return Poll::Ready(Some(Ok(Bytes::from_static(b": ping\n\n"))));
        }
        Poll::Pending
    }
}

impl Drop for SseBody {
    fn drop(&mut self) {
        self.open.fetch_sub(1, Ordering::SeqCst);
        if let Some(f) = self.on_drop.take() {
            f();
        }
    }
}

/// Opens a stream: the sender, and the response that carries it. `open` counts live streams;
/// `on_drop` runs when the client goes away.
pub fn open_stream(
    heartbeat: Duration,
    open: &Arc<AtomicUsize>,
    on_drop: Option<OnDrop>,
) -> (SseSender, Response) {
    let (tx, rx) = unbounded_channel();
    let queued = Arc::new(AtomicUsize::new(0));
    let closed = Arc::new(AtomicBool::new(false));
    open.fetch_add(1, Ordering::SeqCst);
    let body = SseBody {
        rx,
        queued: Arc::clone(&queued),
        closed: Arc::clone(&closed),
        beat: interval_at(Instant::now() + heartbeat, heartbeat),
        open: Arc::clone(open),
        on_drop,
    };
    let res = bytes_reply(
        200,
        Body::from_stream(body),
        &[
            (header::CONTENT_TYPE, "text/event-stream; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache, no-transform"),
            (header::HeaderName::from_static("x-accel-buffering"), "no"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
    );
    (SseSender { tx, queued, closed }, res)
}
