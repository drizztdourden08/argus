//! The mux stream, one connection per browser (wire contract in `sse/mux`):
//!
//! - `GET /api/mux` opens the stream; its first frame is `hello {conn, v}`
//! - `POST /api/mux/:conn {ops}` applies up to 50 ops in order and answers
//!   `{results: [{ok: true} | {error: {code, message}}]}`, one per op; 404 for an unknown
//!   connection, 400 for a body that is not a non-empty list of ops

use argus_core::ApiError;
use serde_json::json;

use super::db::batch_ops;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

async fn open(ctx: Ctx) -> Reply {
    let svc = &ctx.svc;
    svc.mux
        .open(&svc.fanout, svc.heartbeat, &svc.streams, svc.wake.clone())
}

async fn post(ctx: Ctx) -> Reply {
    let conn = ctx
        .svc
        .mux
        .get(ctx.param("conn"))
        .ok_or_else(|| ApiError::not_found("unknown mux connection"))?;
    let body = ctx.json().await?;
    let ops = batch_ops(&body)?;
    let results = conn.apply(&ctx.svc, ops)?;
    Ok(json_reply(200, &json!({ "results": results })))
}

/// Registers the mux routes.
pub fn register(r: &mut Router) {
    r.get("/api/mux", open);
    r.post("/api/mux/:conn", post);
}
