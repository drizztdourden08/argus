//! The Claude accounts, a mount of [`AccountsService`]:
//!
//! - `GET /api/accounts` is `{accounts, config}`: the known accounts and the settings without
//!   them
//! - `POST /api/accounts/usage/refresh` reads every account's usage now and answers
//!   `{results}` with one entry per account
//! - `GET /api/accounts/settings` is the whole settings object, accounts included
//! - `GET /api/accounts/compare` is the ordered rows of folders and chats of every account
//! - `POST /api/accounts/sync {dryRun}` plans (default) or applies the chat and folder sync and
//!   answers the summary; 409 while a run is active
//! - `POST /api/accounts/sync/undo {backupDir?}` puts a backup back, the last one by default
//! - `POST /api/accounts/settings {<keys>}` replaces the keys sent, saves `config.json` and
//!   answers the whole settings object

use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use serde_json::{Value, json};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::AccountsService;
use crate::services::accounts::save;
use crate::services::accounts_sync::SYNC_BUSY;
use crate::{Ctx, RouteOpts, Router};

const SETTINGS_BYTES: usize = 64 * 1024;

fn service(ctx: &Ctx) -> Result<Arc<AccountsService>, ApiError> {
    ctx.svc
        .accounts
        .clone()
        .ok_or_else(|| ApiError::unavailable("the accounts service is not running"))
}

async fn listing(ctx: Ctx) -> Reply {
    let accounts = service(&ctx)?;
    let config = accounts.config();
    let mut view = config.to_json();
    if let Some(object) = view.as_object_mut() {
        object.remove("accounts");
    }
    let body = json!({ "accounts": config.accounts, "config": view });
    Ok(json_reply(200, &body))
}

async fn refresh(ctx: Ctx) -> Reply {
    let results = service(&ctx)?
        .poll_now()
        .await
        .map_err(|e| ApiError::new(ErrorCode::Internal, e))?;
    let view: Vec<Value> = results
        .iter()
        .map(|r| json!({ "key": r.key, "usage": r.usage, "burnPerMin": r.burn_per_min }))
        .collect();
    Ok(json_reply(200, &json!({ "results": view })))
}

async fn settings(ctx: Ctx) -> Reply {
    Ok(json_reply(200, &service(&ctx)?.config().to_json()))
}

async fn update(ctx: Ctx) -> Reply {
    let accounts = service(&ctx)?;
    let body: Value = ctx.json().await?;
    let config = accounts.update(&body).map_err(ApiError::bad_request)?;
    save(&ctx.svc, &config).map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?;
    Ok(json_reply(200, &config.to_json()))
}

fn sync_error(e: String) -> ApiError {
    if e == SYNC_BUSY {
        ApiError::new(ErrorCode::Conflict, e)
    } else {
        ApiError::new(ErrorCode::Internal, e)
    }
}

async fn compare(ctx: Ctx) -> Reply {
    let accounts = service(&ctx)?;
    let rows = tokio::task::spawn_blocking(move || accounts.compare())
        .await
        .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?
        .map_err(|e| ApiError::new(ErrorCode::Internal, e))?;
    Ok(json_reply(200, &json!(rows)))
}

async fn sync(ctx: Ctx) -> Reply {
    let accounts = service(&ctx)?;
    let body: Value = ctx.json().await?;
    let dry_run = body.get("dryRun").and_then(Value::as_bool).unwrap_or(true);
    if accounts.sync_active() {
        return Err(sync_error(SYNC_BUSY.to_owned()));
    }
    let summary = tokio::task::spawn_blocking(move || accounts.sync(dry_run))
        .await
        .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?
        .map_err(sync_error)?;
    Ok(json_reply(200, &json!(summary)))
}

async fn undo(ctx: Ctx) -> Reply {
    let accounts = service(&ctx)?;
    let body: Value = ctx.json().await?;
    let dir = body
        .get("backupDir")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let done = tokio::task::spawn_blocking(move || accounts.undo_sync(dir.as_deref()))
        .await
        .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?
        .map_err(sync_error)?;
    let report = json!({ "restored": done.report.restored, "movedAside": done.report.moved_aside });
    Ok(json_reply(
        200,
        &json!({ "backupDir": done.backup_dir, "report": report }),
    ))
}

/// Registers the accounts routes.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        max_bytes: SETTINGS_BYTES,
        ..RouteOpts::default()
    };
    r.get("/api/accounts", listing);
    r.post("/api/accounts/usage/refresh", refresh);
    r.get("/api/accounts/compare", compare);
    r.post("/api/accounts/sync", sync);
    r.post("/api/accounts/sync/undo", undo);
    r.get("/api/accounts/settings", settings);
    r.add("POST", "/api/accounts/settings", opts, update);
}
