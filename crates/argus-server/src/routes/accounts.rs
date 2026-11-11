//! The Claude accounts, a mount of [`AccountsService`]:
//!
//! - `GET /api/accounts` is `{accounts, config, appVersion}`: the known accounts, the settings
//!   without them and the installed Claude version (null when unreadable)
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
use crate::services::accounts_switch::app_version;
use crate::services::accounts_sync::SYNC_BUSY;
use crate::{Ctx, RouteOpts, Router};

const SETTINGS_BYTES: usize = 64 * 1024;

/// Refuses what changes Claude's data, logins or tasks in shadow mode and in a preview.
///
/// # Errors
/// 403 `read_only` in shadow mode and in a preview.
pub(crate) fn outward(ctx: &Ctx, what: &str) -> Result<(), ApiError> {
    if ctx.svc.shadow {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            format!("shadow or preview mode: Argus does not {what}; accounts are read-only"),
        ));
    }
    Ok(())
}

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
    let service = Arc::clone(&accounts);
    let version = tokio::task::spawn_blocking(move || app_version(&service))
        .await
        .ok()
        .flatten();
    let body = json!({ "accounts": config.accounts, "config": view, "appVersion": version });
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
    // A preview home is a copy, so its settings may change; a shadow home writes nothing.
    if !ctx.svc.board_writes {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus does not write the accounts settings",
        ));
    }
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
    if !dry_run {
        outward(&ctx, "apply a sync")?;
    }
    if accounts.sync_active() {
        return Err(sync_error(SYNC_BUSY.to_owned()));
    }
    let summary = tokio::task::spawn_blocking(move || accounts.try_sync(dry_run))
        .await
        .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?
        .map_err(sync_error)?;
    Ok(json_reply(200, &json!(summary)))
}

async fn undo(ctx: Ctx) -> Reply {
    outward(&ctx, "undo a sync")?;
    let accounts = service(&ctx)?;
    let body: Value = ctx.json().await?;
    let dir = body
        .get("backupDir")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let force = body.get("force").and_then(Value::as_bool).unwrap_or(false);
    let done = tokio::task::spawn_blocking(move || accounts.undo_sync(dir.as_deref(), force))
        .await
        .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?
        .map_err(sync_error)?;
    let report = json!({
        "restored": done.report.restored,
        "movedAside": done.report.moved_aside,
        "saved": done.report.saved,
        "kept": done.report.kept,
    });
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
