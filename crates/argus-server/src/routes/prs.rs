//! Local pull requests (`docs/local-remote.md`): the board's `prs` collection is written here and
//! by `coord pr`; the pages only read it from the board stream.
//!
//! - `POST /api/p/:key/prs {title, body?, head, base?, headCommit?, item?, by?}` opens one
//! - `PATCH /api/p/:key/prs/:id {action?, ...}` changes one: `action` is `update` (the default;
//!   `headCommit`, `title`, `body`), `merge` (`mergeCommit`, `mergedBy?`) or `close`
//!
//! An answer is `{pr, outcome, seq}` with the record as the board holds it. A refusal of the board
//! is a 400 with its sentence, an unknown pull request a 404.

use argus_core::ApiError;
use serde_json::{Map, Value, json};

use super::params::{key, sync};
use super::write::{actor, object_body, writable};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

const OPEN_FIELDS: [&str; 6] = ["title", "body", "head", "base", "headCommit", "item"];
const UPDATE_FIELDS: [&str; 3] = ["headCommit", "title", "body"];
const MERGE_FIELDS: [&str; 2] = ["mergeCommit", "mergedBy"];

/// The fields of `body` that are named, as the `set` of an op.
fn picked(body: &Value, names: &[&str]) -> Value {
    let set: Map<String, Value> = names
        .iter()
        .filter_map(|n| body.get(*n).map(|v| ((*n).to_owned(), v.clone())))
        .collect();
    Value::Object(set)
}

fn kind_of(body: &Value) -> Result<(&'static str, &'static [&'static str]), ApiError> {
    match body["action"].as_str().unwrap_or("update") {
        "update" => Ok(("pr-update", &UPDATE_FIELDS)),
        "merge" => Ok(("pr-merge", &MERGE_FIELDS)),
        "close" => Ok(("pr-close", &[])),
        other => Err(ApiError::bad_request(format!(
            "action must be update, merge or close, not \"{other}\""
        ))),
    }
}

fn unknown_is_404(e: ApiError) -> ApiError {
    if e.message.starts_with("no pull request") {
        ApiError::not_found(e.message)
    } else {
        e
    }
}

async fn open(ctx: Ctx) -> Reply {
    writable(&ctx)?;
    let body = object_body(&ctx).await?;
    let op = json!({ "kind": "pr-open", "set": picked(&body, &OPEN_FIELDS) });
    let done = sync(&ctx)?
        .apply_pr(&key(&ctx)?, op, &actor(&ctx, &body))
        .await?;
    Ok(json_reply(
        200,
        &json!({ "pr": done.pr, "outcome": done.outcome, "seq": done.seq }),
    ))
}

async fn change(ctx: Ctx) -> Reply {
    writable(&ctx)?;
    let body = object_body(&ctx).await?;
    let (kind, fields) = kind_of(&body)?;
    let target = json!({ "c": "prs", "id": ctx.param("id") });
    let op = json!({ "kind": kind, "target": target, "set": picked(&body, fields) });
    let done = sync(&ctx)?
        .apply_pr(&key(&ctx)?, op, &actor(&ctx, &body))
        .await
        .map_err(unknown_is_404)?;
    Ok(json_reply(
        200,
        &json!({ "pr": done.pr, "outcome": done.outcome, "seq": done.seq }),
    ))
}

/// Registers the pull request routes.
pub fn register(r: &mut Router) {
    r.post("/api/p/:key/prs", open);
    r.add("PATCH", "/api/p/:key/prs/:id", RouteOpts::default(), change);
}
