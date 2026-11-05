//! `GET /api/p/:key/icon`: the image a board wears as its icon, decoded from the data URL the board
//! keeps. The card's url carries `?v=<hash>`, so the answer is cached for good. A board whose icon
//! is not an image, or that has none, is a 404.

use argus_board::ops::icon_bytes;
use argus_core::ApiError;
use axum::http::header;

use super::blocking::blocking;
use super::params::key;
use crate::reply::bytes_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

const POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'";
const CACHE: &str = "public, max-age=31536000, immutable";

async fn serve(ctx: Ctx) -> Reply {
    let key = key(&ctx)?;
    let svc = ctx.svc.clone();
    let stored = blocking(move || {
        let board = svc.boards().board(&key)?;
        board["icon"].as_str().map(str::to_owned)
    })
    .await?;
    let (kind, bytes) = stored
        .as_deref()
        .and_then(icon_bytes)
        .ok_or_else(|| ApiError::not_found("this board has no image icon"))?;
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

/// Registers the board icon route.
pub fn register(r: &mut Router) {
    r.get("/api/p/:key/icon", serve);
}
