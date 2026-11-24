//! The tell of a thread that read its own inbox (`coord inbox`, best effort):
//!
//! - `POST /api/p/:key/inbox/read {thread, deps?, actor?}` stamps the read for the stall watch and
//!   acknowledges the dependency requests among `deps` that are open and asked of this project.
//!   The thread must be on the board (404). The answer is `{readAt, acked}`, the ids acknowledged
//!   now. In shadow mode it answers the same and writes nothing.

use argus_core::ApiError;
use argus_core::clock::now_iso;
use argus_core::lead::canonical;
use argus_wake::external::valid_thread_id;
use argus_wake::service::driver::lock;
use serde_json::{Value, json};

use super::blocking::blocking;
use super::params::key;
use super::wake::service;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::deps::ack;
use crate::{Ctx, Router};

/// The most dependencies one read acknowledges.
const MAX_DEPS: usize = 50;

fn thread_of(body: &Value) -> Result<String, ApiError> {
    body["thread"]
        .as_str()
        .filter(|t| valid_thread_id(t))
        .map(|t| canonical(t).into_owned())
        .ok_or_else(|| ApiError::bad_request("thread must be a thread id"))
}

fn deps_of(body: &Value) -> Vec<String> {
    body["deps"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|id| !id.is_empty())
        .take(MAX_DEPS)
        .map(str::to_owned)
        .collect()
}

fn on_board(ctx: &Ctx, key: &str, thread: &str) -> bool {
    let boards = ctx.svc.boards();
    boards
        .board(key)
        .or_else(|| boards.fresh(key))
        .is_some_and(|b| b["threads"].get(thread).is_some_and(|t| !t.is_null()))
}

async fn read(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let project = key(&ctx)?;
    let body = ctx.json().await?;
    let thread = thread_of(&body)?;
    if !on_board(&ctx, &project, &thread) {
        return Err(ApiError::not_found(format!(
            "no thread {thread} on the board of {project}"
        )));
    }
    let actor = body["actor"]
        .as_str()
        .filter(|a| !a.is_empty())
        .map_or_else(|| format!("{project}/{thread}"), str::to_owned);
    let (scope, name) = (project.clone(), thread.clone());
    blocking(move || lock(&wake).inbox_read(&scope, &name)).await?;
    let acked = ack(&ctx.svc, project, deps_of(&body), actor).await;
    Ok(json_reply(
        200,
        &json!({ "readAt": now_iso(), "acked": acked }),
    ))
}

/// Registers the inbox route.
pub fn register(r: &mut Router) {
    r.post("/api/p/:key/inbox/read", read);
}
