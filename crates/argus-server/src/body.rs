//! Request bodies up to a limit: a declared length over it is refused before keeping anything,
//! a body that grows past it is refused as it arrives. The rest of a refused body is drained (as
//! Node does), so the client reads the 413 instead of a reset connection. A body nobody read (an
//! early refusal) is discarded after its answer, as Node's `req._dump()` does.

use std::time::Duration;

use argus_core::{ApiError, ErrorCode};
use axum::body::{Body, HttpBody};
use axum::http::HeaderMap;
use axum::http::header::CONTENT_LENGTH;
use bytes::{Bytes, BytesMut};
use http_body_util::BodyExt;

/// The most of an unread body discarded after an early answer.
const DISCARD_MAX: usize = 64 * 1024 * 1024;
/// How long a client may take to send the rest of an unread body.
const DISCARD_TIME: Duration = Duration::from_secs(10);

fn too_large(message: String) -> ApiError {
    ApiError::new(ErrorCode::TooLarge, message)
}

/// Reads and drops frames until the body ends, fails or passes `max` bytes.
async fn drain_up_to(mut body: Body, max: usize) {
    let mut seen = 0;
    while seen <= max {
        match body.frame().await {
            Some(Ok(frame)) => seen += frame.data_ref().map_or(0, Bytes::len),
            _ => return,
        }
    }
}

async fn drain(body: Body) {
    drain_up_to(body, usize::MAX).await;
}

/// Keeps reading a body nobody read while its answer goes out, then drops it. The connection
/// stays open until the client has sent it all, so closing it never discards unread bytes (a
/// TCP reset, which on Windows also throws away the answer the client has not read yet). A body
/// past [`DISCARD_MAX`] or slower than [`DISCARD_TIME`] is given up.
pub(crate) fn discard(body: Body) {
    if body.is_end_stream() {
        return;
    }
    if let Ok(rt) = tokio::runtime::Handle::try_current() {
        rt.spawn(tokio::time::timeout(
            DISCARD_TIME,
            drain_up_to(body, DISCARD_MAX),
        ));
    }
}

/// Reads at most `max` bytes.
///
/// # Errors
/// 413 `too_large` past the limit, 400 when the client aborts.
pub async fn read_limited(headers: &HeaderMap, body: Body, max: usize) -> Result<Bytes, ApiError> {
    let declared = headers
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    if let Some(n) = declared.filter(|n| *n > max as u64) {
        drain(body).await;
        return Err(too_large(format!("body is {n} bytes, the limit is {max}")));
    }
    let mut body = body;
    let mut out = BytesMut::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|_| ApiError::bad_request("request aborted"))?;
        if let Ok(data) = frame.into_data() {
            if out.len() + data.len() > max {
                drain(body).await;
                return Err(too_large(format!("body is over the limit of {max} bytes")));
            }
            out.extend_from_slice(&data);
        }
    }
    Ok(out.freeze())
}
