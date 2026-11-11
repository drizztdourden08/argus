//! The sync review routes, mounts of [`crate::services::accounts_review`]:
//!
//! - `GET /api/accounts/review` is every folder, chat and sub-thread of the accounts' home
//!   scopes, each difference with its choices and the default the automatic sync would take
//! - `GET /api/accounts/cleanup?days=N` is the archived chats untouched for more than N days
//!   (default 30), per account
//! - `POST /api/accounts/review/apply {decisions, edits, closeDesktop}` starts the run of the
//!   owner's choices and edits and answers 202 `{id}` at once; the `run` doc and the
//!   `SyncSteps` docs follow it. 400 for a body that does not parse, 409 while a sync, a review
//!   or a switch runs, 403 `read_only` in shadow mode and in a preview

use argus_accounts::sync::review::Request;
use argus_core::{ApiError, ErrorCode};
use serde_json::{Value, json};

use crate::services::accounts_review::{CLEANUP_DAYS, SWITCH_ACTIVE, prepare};
use crate::services::{AccountsService, accounts_sync::SYNC_BUSY};
use crate::{Ctx, RouteOpts, Router, reply::json_reply, router::Reply, routes::accounts::outward};

const APPLY_BYTES: usize = 4 * 1024 * 1024;

fn service(ctx: &Ctx) -> Result<std::sync::Arc<AccountsService>, ApiError> {
    ctx.svc
        .accounts
        .clone()
        .ok_or_else(|| ApiError::unavailable("the accounts service is not running"))
}

fn internal(e: String) -> ApiError {
    ApiError::new(ErrorCode::Internal, e)
}

async fn review(ctx: Ctx) -> Reply {
    let accounts = service(&ctx)?;
    let review = tokio::task::spawn_blocking(move || accounts.review())
        .await
        .map_err(|e| internal(e.to_string()))?
        .map_err(internal)?;
    Ok(json_reply(200, &json!(review)))
}

async fn cleanup(ctx: Ctx) -> Reply {
    let accounts = service(&ctx)?;
    let days = match ctx.query("days") {
        Some(text) => text
            .parse::<u32>()
            .map_err(|_| ApiError::bad_request("`days` must be a whole number of days"))?,
        None => CLEANUP_DAYS,
    };
    let found = tokio::task::spawn_blocking(move || accounts.cleanup(days))
        .await
        .map_err(|e| internal(e.to_string()))?
        .map_err(internal)?;
    Ok(json_reply(200, &json!(found)))
}

async fn apply(ctx: Ctx) -> Reply {
    outward(&ctx, "apply a sync review")?;
    let accounts = service(&ctx)?;
    let body: Value = ctx.json().await?;
    let close = body
        .get("closeDesktop")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let request: Request = serde_json::from_value(body)
        .map_err(|e| ApiError::bad_request(format!("not a review: {e}")))?;
    let run = prepare(&accounts, request, close).map_err(|e| {
        if e == SYNC_BUSY || e == SWITCH_ACTIVE {
            ApiError::new(ErrorCode::Conflict, e)
        } else {
            internal(e)
        }
    })?;
    let id = run.id().to_owned();
    let log = ctx.svc.log.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = run.run() {
            log.warn("accounts review run ended", json!({ "reason": e }));
        }
    });
    Ok(json_reply(202, &json!({ "id": id })))
}

/// Registers the review routes.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        max_bytes: APPLY_BYTES,
        ..RouteOpts::default()
    };
    r.get("/api/accounts/review", review);
    r.get("/api/accounts/cleanup", cleanup);
    r.add("POST", "/api/accounts/review/apply", opts, apply);
}
