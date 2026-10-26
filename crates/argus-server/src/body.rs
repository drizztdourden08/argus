//! Request bodies up to a limit: a declared length over it is refused before keeping anything,
//! a body that grows past it is refused as it arrives. The rest of a refused body is drained (as
//! Node does), so the client reads the 413 instead of a reset connection.

use argus_core::{ApiError, ErrorCode};
use axum::body::Body;
use axum::http::HeaderMap;
use axum::http::header::CONTENT_LENGTH;
use bytes::{Bytes, BytesMut};
use http_body_util::BodyExt;

fn too_large(message: String) -> ApiError {
    ApiError::new(ErrorCode::TooLarge, message)
}

async fn drain(mut body: Body) {
    while let Some(Ok(_)) = body.frame().await {}
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
