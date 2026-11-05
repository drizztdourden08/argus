//! The answer of an image route: the bytes of an image stored as a data URL, with the type, the
//! cache for good and the safety headers.

use argus_board::ops::icon_bytes;
use argus_core::ApiError;
use axum::http::header;

use crate::reply::bytes_reply;
use crate::router::Reply;

const POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'";
const CACHE: &str = "public, max-age=31536000, immutable";

/// The image kept in `stored` as a 200, or a 404 that says `missing` when it is none or no image.
pub(super) fn image_reply(stored: Option<&str>, missing: &str) -> Reply {
    let (kind, bytes) = stored
        .and_then(icon_bytes)
        .ok_or_else(|| ApiError::not_found(missing))?;
    let len = bytes.len().to_string();
    Ok(bytes_reply(
        200,
        bytes,
        &[
            (header::CONTENT_TYPE, kind),
            (header::CONTENT_LENGTH, len.as_str()),
            (header::CACHE_CONTROL, CACHE),
            (header::CONTENT_SECURITY_POLICY, POLICY),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
    ))
}
