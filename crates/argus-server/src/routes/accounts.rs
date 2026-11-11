//! The Claude accounts, a mount of [`AccountsService`]:
//!
//! - `GET /api/accounts` is `{accounts, config}`: the known accounts and the settings without
//!   them
//! - `POST /api/accounts/usage/refresh` reads every account's usage now and answers
//!   `{results}` with one entry per account
//! - `GET /api/accounts/settings` is the whole settings object, accounts included
//! - `POST /api/accounts/settings {<keys>}` replaces the keys sent, saves `config.json` and
//!   answers the whole settings object

use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use serde_json::{Value, json};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::AccountsService;
use crate::services::accounts::save;
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

/// Registers the accounts routes.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        max_bytes: SETTINGS_BYTES,
        ..RouteOpts::default()
    };
    r.get("/api/accounts", listing);
    r.post("/api/accounts/usage/refresh", refresh);
    r.get("/api/accounts/settings", settings);
    r.add("POST", "/api/accounts/settings", opts, update);
}
