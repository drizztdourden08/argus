//! The build cache of Brock and Tessera (Brock's `docs/build-cache.md`; Argus owns usage tracking and pruning):
//!
//! - `GET /api/cache` is `{root, exists, graceDays, rows, totals, problems, lastPrune, shadow}`: one row per cached
//!   version with every project and worktree that resolves it in its lockfile, the totals, and the last prune of this
//!   run (`null` before one)
//! - `POST /api/cache/prune` prunes now and answers what it deleted: `{at, deleted, tmpDeleted, freedBytes, refused,
//!   failed}`. 403 `read_only` in shadow mode, 409 `conflict` while the worktrees were not scanned yet or a lockfile
//!   could not be read (nothing is deleted then)

use std::sync::Arc;

use serde_json::{Value, json, to_value};

use super::blocking::blocking;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::cache;
use crate::{Ctx, RouteOpts, Router};

const SMALL: usize = 1024;

async fn show(ctx: Ctx) -> Reply {
    let svc = Arc::clone(&ctx.svc);
    let report = blocking(move || cache::report(&svc)).await??;
    let mut body = to_value(&report).unwrap_or(Value::Null);
    let last = ctx.svc.cache.as_ref().and_then(|c| c.last_prune());
    body["lastPrune"] = json!(last);
    body["shadow"] = json!(ctx.svc.shadow);
    Ok(json_reply(200, &body))
}

async fn prune(ctx: Ctx) -> Reply {
    let svc = Arc::clone(&ctx.svc);
    let outcome = blocking(move || cache::prune(&svc, true)).await??;
    Ok(json_reply(200, &json!(outcome)))
}

/// Registers the build cache routes.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    r.get("/api/cache", show);
    r.add("POST", "/api/cache/prune", small, prune);
}
