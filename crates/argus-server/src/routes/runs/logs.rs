//! The lines of a run, by cursor or by tail.

use argus_launchers::LogOptions;
use serde_json::json;

use super::{ids, refusal, service};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

/// A query number: whole, or not given.
pub(super) fn number(ctx: &Ctx, name: &str) -> Option<i64> {
    ctx.query(name).and_then(|n| n.trim().parse().ok())
}

async fn lines(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    let options = LogOptions {
        stream: ctx.query("stream").map(str::to_owned),
        tail: number(&ctx, "tail"),
        after: number(&ctx, "after"),
        limit: number(&ctx, "limit"),
    };
    let page = runs.logs(&key, &rid, &options).map_err(|e| refusal(&e))?;
    Ok(json_reply(
        200,
        &json!({ "lines": page.lines, "next": page.next, "dropped": page.dropped }),
    ))
}

pub(super) fn register(r: &mut Router) {
    r.get("/api/p/:key/runs/:rid/logs", lines);
}
