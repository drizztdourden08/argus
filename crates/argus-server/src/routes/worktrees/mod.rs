//! Removing a worktree through Argus (DESIGN 13.12; the crate has every guard).
//!
//! - `POST /api/p/:key/worktrees/remove {id, branch?, discard?, ownerEntry?, reason?, head?,
//!   dirty?, newestAt?, confirmYoung?, by?}` removes one worktree, or starts the deletion of one
//!   leftover folder: `{removed, path, branch, deletedBranch, ..., text}`
//! - `POST /api/p/:key/worktrees/prune {id?, reason?, by?}` prunes one gone worktree, or every
//!   gone one of the project
//! - `POST /api/p/:key/worktrees/release {id, reason?, by}` a thread's release: after a fresh
//!   look at that one worktree (no scan of the project, so it never waits for a running removal), nothing may use it and nothing may be lost, then the removal (the branch too when
//!   merged) is filed as a request of the thread and answered at once: `{id, status, text,
//!   request}`; the project's worker removes it and the result comes to the thread's inbox
//!   (`docs/worktree-hygiene.md`)
//! - `POST /api/p/:key/worktrees/approve {id | ids, reason?, by}` the owning lead or the Overlord approves removing
//!   worktrees that hold work and nothing is attached to: Argus files ONE `approved` request (a full backup of each,
//!   then the removal, the branches kept) and answers at once without reading git. With `id`: `{id, status, text,
//!   request}`, or the refusal. With `ids`: `{queued, refused: [{id, code, text}]}` (`queued` is null when none
//!   passed); the ids that pass go ahead. The result comes to the approver's inbox
//! - `POST /api/p/:key/worktrees/keep {id, note, until?, askedBy?, clear?, by, byProject}` the Overlord's exception list: a
//!   worktree on it is never removed or reported (`until` an ISO date or milliseconds; `clear` takes it off). Anyone
//!   else is refused with 403 `forbidden` and told to ask the Overlord: `{id, text}`
//! - `POST /api/p/:key/worktrees/leftovers/remove {ids?, olderThanDays?, confirmYoung?, reason?,
//!   by?}` queues every old leftover folder for deletion in the background
//! - `POST /api/p/:key/worktrees/requests {targets? | finished? | allLeftovers?, branch?, discard?,
//!   ownerEntry?, reason?, olderThanDays?, idleMinutes? (finished only), by}` only records a request (a thread asks, the
//!   server's one worker removes through the same guards and tells the thread's inbox):
//!   `{id, status, text, request}`; `GET` lists them newest first: `{requests}`
//!
//! Who: the owner's page (discarding work allowed, with the backup), or a registered thread of
//! the project named in `by`. The page is told by the authenticated session, never by the body.
//! Refusals carry the crate's codes: 403 `forbidden`, 404 `not_found`, 409 `conflict`, 423
//! `in_use`, 500 `backup_failed` and `git_failed`. Shadow mode answers `read_only`.

use std::future::Future;
use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use argus_worktrees::{Caller, Service as WorktreeService, WtError};
use axum::response::Response;
use serde_json::{Value, json};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::blocking::blocking;
use crate::routes::params;
use crate::services::worktrees::publish;
use crate::{AuthKind, Ctx, RouteOpts, Router};

mod hygiene;
mod leftovers;
mod remove;
mod requests;

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

/// The reason, its white space folded and cut short.
fn reason(body: &Value) -> Option<String> {
    let mut out = String::new();
    for word in body["reason"].as_str()?.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
        if out.chars().count() >= REASON_MAX {
            break;
        }
    }
    let cut: String = out.chars().take(REASON_MAX).collect();
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

/// What the routes share: the checks, the body, the call on a thread that may block, the
/// rows written again, and the answer.
async fn serve<I: Send + 'static, T: Send + 'static>(
    ctx: Ctx,
    input: fn(&Ctx, &str, &Value) -> I,
    call: fn(&WorktreeService, &I) -> Result<T, WtError>,
    render: fn(T) -> Value,
) -> Reply {
    let service = service(&ctx)?;
    writable(&ctx)?;
    let key = params::key(&ctx)?;
    let body = ctx.json().await?;
    let input = input(&ctx, &key, &body);
    let svc = Arc::clone(&ctx.svc);
    let out = blocking(move || {
        let out = call(&service, &input);
        publish(&svc, &service, &key);
        out
    })
    .await?;
    Ok(match out {
        Ok(done) => json_reply(200, &render(done)),
        Err(e) => refusal(&e),
    })
}

fn post<F, Fut>(r: &mut Router, path: &str, max_bytes: usize, f: F)
where
    F: Fn(Ctx) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Reply> + Send + 'static,
{
    let opts = RouteOpts {
        max_bytes,
        ..RouteOpts::default()
    };
    r.add("POST", path, opts, f);
}

/// Registers the worktree routes.
pub fn register(r: &mut Router) {
    remove::register(r);
    hygiene::register(r);
    leftovers::register(r);
    requests::register(r);
}
