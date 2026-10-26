//! Images attached to a run: a raw image body in, the list out.

use argus_launchers::AttachmentInfo;
use serde_json::{Value, json};

use super::{actor, ids, refusal, service, writable};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

/// The largest body a route reads; the configured `attach.maxBytes` applies below it.
const BODY_CEILING: usize = 64 * 1024 * 1024;
const DEFAULT_MAX: usize = 10 * 1024 * 1024;

fn listed(info: &AttachmentInfo) -> Value {
    json!({
        "id": info.id,
        "url": info.url,
        "caption": info.caption,
        "mime": info.mime,
        "bytes": info.bytes,
        "n": info.n,
        "by": info.by,
        "at": info.at,
    })
}

async fn add(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    writable(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    let max = usize::try_from(ctx.svc.config().attach.max_bytes).unwrap_or(DEFAULT_MAX);
    let bytes = ctx.raw(max).await?;
    let mime = ctx.header("content-type").unwrap_or_default();
    let caption = ctx.query("caption").unwrap_or_default();
    let info = runs
        .attach(&key, &rid, &bytes, mime, caption, actor(&ctx))
        .map_err(|e| refusal(&e))?;
    Ok(json_reply(
        200,
        &json!({ "attachment": { "id": info.id, "url": info.url, "caption": info.caption } }),
    ))
}

async fn list(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    let all = runs.attachments(&key, &rid).map_err(|e| refusal(&e))?;
    let rows: Vec<Value> = all.iter().map(listed).collect();
    Ok(json_reply(200, &json!({ "attachments": rows })))
}

pub(super) fn register(r: &mut Router) {
    let images = RouteOpts {
        images: true,
        max_bytes: BODY_CEILING,
        ..RouteOpts::default()
    };
    r.add("POST", "/api/p/:key/runs/:rid/attachments", images, add);
    r.get("/api/p/:key/runs/:rid/attachments", list);
}
