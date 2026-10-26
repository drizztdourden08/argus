//! Removing a worktree through Argus (DESIGN 13.12; the crate has every guard).
//!
//! - `POST /api/p/:key/worktrees/remove {id, branch?, discard?, ownerEntry?, reason?, head?,
//!   dirty?, newestAt?, confirmYoung?, by?}` removes one worktree, or starts the deletion of one
//!   leftover folder: `{removed, path, branch, deletedBranch, ..., text}`
//! - `POST /api/p/:key/worktrees/prune {id?, reason?, by?}` prunes one gone worktree, or every
//!   gone one of the project
//! - `POST /api/p/:key/worktrees/leftovers/remove {ids?, olderThanDays?, confirmYoung?, reason?,
//!   by?}` queues every old leftover folder for deletion in the background
//!
//! Who: the owner's page (discarding work allowed, with the backup), or a registered thread of
//! the project named in `by`. The page is told by the authenticated session, never by the body.
//! Refusals carry the crate's codes: 403 `forbidden`, 404 `not_found`, 409 `conflict`, 423
//! `in_use`, 500 `backup_failed` and `git_failed`. Shadow mode answers `read_only`.

use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use argus_worktrees::{Caller, Service as WorktreeService, WtError};
use axum::response::Response;
use serde_json::{Value, json};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::{AuthKind, Ctx, Router};

mod leftovers;
mod remove;

const REASON_MAX: usize = 200;
const ID_MAX: usize = 400;
const BY_MAX: usize = 60;

fn service(ctx: &Ctx) -> Result<Arc<WorktreeService>, ApiError> {
    ctx.svc
        .worktrees
        .clone()
        .ok_or_else(|| ApiError::unavailable("the worktrees service is not running"))
}

fn writable(ctx: &Ctx) -> Result<(), ApiError> {
    if ctx.svc.shadow {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus removes no worktree",
        ));
    }
    Ok(())
}

/// The owner's page when the session is the page's; otherwise the thread `by` names.
fn caller(ctx: &Ctx, body: &Value) -> Caller {
    if ctx.auth == Some(AuthKind::Page) {
        return Caller::Page;
    }
    let by = body["by"].as_str().unwrap_or_default();
    Caller::Thread(by.chars().take(BY_MAX).collect())
}

fn reason(body: &Value) -> Option<String> {
    let text = body["reason"].as_str()?;
    let folded = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = folded.chars().take(REASON_MAX).collect();
    (!cut.is_empty()).then_some(cut)
}

fn id_of(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(ID_MAX)
        .collect()
}

/// A refusal of the crate as the answer: its own code and status, and the backup it had
/// already written.
fn refusal(e: &WtError) -> Response {
    let mut error = json!({ "code": e.code.as_str(), "message": e.message });
    if let Some(backup) = &e.backup {
        error["details"] = json!({ "backup": backup });
    }
    json_reply(e.code.status(), &json!({ "error": error }))
}

/// Runs a removal on a thread that may block, then writes the rows again.
async fn blocking<T: Send + 'static>(
    ctx: &Ctx,
    key: &str,
    service: Arc<WorktreeService>,
    work: impl FnOnce(&WorktreeService) -> Result<T, WtError> + Send + 'static,
) -> Result<Result<T, WtError>, ApiError> {
    let (svc, key) = (Arc::clone(&ctx.svc), key.to_owned());
    tokio::task::spawn_blocking(move || {
        let out = work(&service);
        crate::services::worktrees::publish(&svc, &service, &key);
        out
    })
    .await
    .map_err(|_| ApiError::internal())
}

fn answer<T>(out: Result<T, WtError>, body: impl FnOnce(T) -> Value) -> Reply {
    Ok(match out {
        Ok(done) => json_reply(200, &body(done)),
        Err(e) => refusal(&e),
    })
}

/// Registers the worktree routes.
pub fn register(r: &mut Router) {
    remove::register(r);
    leftovers::register(r);
}
