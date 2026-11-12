//! The Desktop switch routes, mounts of [`crate::services::accounts_switch`]:
//!
//! - `POST /api/accounts/switch {to, handover}` checks and starts a switch and answers 202
//!   `{id}` at once; the `switch` doc follows it. 409 while one runs, 400 when it cannot start
//! - `POST /api/accounts/switch/anyway` stops waiting for the handovers
//! - `POST /api/accounts/switch/cancel` cancels a switch that has not closed Claude yet; answers
//!   `{cancelled}`
//! - `POST /api/accounts/login/save {key?}` saves the login Desktop is on, for its account, and
//!   answers `{savedAt}`

use argus_core::{ApiError, ErrorCode};
use serde_json::{Value, json};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::accounts::outward;
use crate::services::accounts_review;
use crate::services::accounts_rules::{cancel_countdown, end_countdown};
use crate::services::accounts_switch::{
    Origin, SWITCH_BUSY, cancel, prepare, request_anyway, save_login,
};
use crate::{Ctx, Router};

fn refused(e: String) -> ApiError {
    if e == SWITCH_BUSY {
        ApiError::new(ErrorCode::Conflict, e)
    } else {
        ApiError::bad_request(e)
    }
}

async fn start(ctx: Ctx) -> Reply {
    outward(&ctx, "switch Claude Desktop")?;
    let body: Value = ctx.json().await?;
    let to = body
        .get("to")
        .and_then(Value::as_str)
        .ok_or_else(|| ApiError::bad_request("`to` must be an account key"))?;
    let handover = body
        .get("handover")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let switch = prepare(&ctx.svc, to, Origin::Owner, handover).map_err(refused)?;
    if let Some(accounts) = &ctx.svc.accounts {
        end_countdown(accounts).await;
    }
    let id = switch.id().to_owned();
    let log = ctx.svc.log.clone();
    tokio::spawn(async move {
        if let Err(e) = switch.run().await {
            log.warn("accounts switch ended", json!({ "reason": e }));
        }
    });
    Ok(json_reply(202, &json!({ "id": id })))
}

async fn anyway(ctx: Ctx) -> Reply {
    outward(&ctx, "switch Claude Desktop")?;
    Ok(json_reply(200, &json!({ "requested": request_anyway() })))
}

async fn cancelling(ctx: Ctx) -> Reply {
    let countdown = match &ctx.svc.accounts {
        Some(accounts) => cancel_countdown(accounts).await,
        None => false,
    };
    Ok(json_reply(
        200,
        &json!({ "cancelled": countdown || cancel() || accounts_review::cancel() }),
    ))
}

async fn save(ctx: Ctx) -> Reply {
    outward(&ctx, "save a Desktop login")?;
    let accounts = ctx
        .svc
        .accounts
        .clone()
        .ok_or_else(|| ApiError::unavailable("the accounts service is not running"))?;
    let body: Value = ctx.json().await?;
    let key = body.get("key").and_then(Value::as_str).map(str::to_owned);
    let svc = ctx.svc.clone();
    let saved = tokio::task::spawn_blocking(move || save_login(&svc, &accounts, key.as_deref()))
        .await
        .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?
        .map_err(ApiError::bad_request)?;
    Ok(json_reply(200, &json!({ "savedAt": saved })))
}

/// Registers the switch routes.
pub fn register(r: &mut Router) {
    r.post("/api/accounts/switch", start);
    r.post("/api/accounts/switch/anyway", anyway);
    r.post("/api/accounts/switch/cancel", cancelling);
    r.post("/api/accounts/login/save", save);
}
