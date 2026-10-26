//! Request bodies up to a limit: a declared length over it is refused before reading, a body
//! that grows past it is refused as it arrives.

use argus_core::{ApiError, ErrorCode};
use axum::body::Body;
use axum::http::HeaderMap;
use axum::http::header::CONTENT_LENGTH;
use bytes::{Bytes, BytesMut};
use http_body_util::BodyExt;

fn too_large(message: String) -> ApiError {
    ApiError::new(ErrorCode::TooLarge, message)
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
        return Err(too_large(format!("body is {n} bytes, the limit is {max}")));
    }
    let mut body = body;
    let mut out = BytesMut::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|_| ApiError::bad_request("request aborted"))?;
        if let Ok(data) = frame.into_data() {
            if out.len() + data.len() > max {
                return Err(too_large(format!("body is over the limit of {max} bytes")));
            }
            out.extend_from_slice(&data);
        }
    }
    Ok(out.freeze())
}
