//! The hosted Overlord's settings, for the owner's page only (I-200):
//!
//! - `GET /api/overlord/settings` is `{model, effort, permissionMode, choices: {models: [{id,
//!   label}], efforts, permissionModes: [{id, label}]}}` plus `cwd`, `claudeExe`, `budgetUsd`,
//!   `restartMax` and `approvalTimeoutMin`, each `{value, source}` with source `set`, `board` or
//!   `default` (I-207)
//! - `POST /api/overlord/settings` takes any of those fields (an empty text resets one to its
//!   default), checks each (400 on a bad value, a missing folder or file, a number out of
//!   range), saves it in `config.overlord`, restarts a running child on the same session when a
//!   setting changed and answers the same shape

use argus_core::{ApiError, ErrorCode};
use argus_overlord::session::choices::Change;
use serde_json::Value;

use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::settings::{SettingsFault, apply, body};
use crate::services::overlord::settings_limits::Limits;
use crate::{Ctx, RouteOpts, Router};

const SMALL: usize = 8 * 1024;

/// A text field of the request; missing or null is no change.
fn text_field(request: &Value, name: &str) -> Result<Option<String>, ApiError> {
    match request.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ApiError::bad_request(format!("{name} must be text"))),
    }
}

fn change_of(request: &Value) -> Result<Change, ApiError> {
    Ok(Change {
        model: text_field(request, "model")?,
        effort: text_field(request, "effort")?,
        permission_mode: text_field(request, "permissionMode")?,
    })
}

fn refusal(fault: SettingsFault) -> ApiError {
    match fault {
        SettingsFault::Bad(bad) => ApiError::bad_request(bad.sentence()),
        SettingsFault::Invalid(sentence) => ApiError::bad_request(sentence),
        SettingsFault::Storage => {
            ApiError::new(ErrorCode::Internal, "the settings could not be saved")
        }
        SettingsFault::Restart(refused) => ApiError::new(ErrorCode::Conflict, refused.sentence()),
    }
}

async fn show(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    Ok(json_reply(200, &body(&ctx.svc)))
}

async fn change(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let request = ctx.json().await?;
    let wanted = change_of(&request)?;
    let limits = Limits::parse(&request).map_err(ApiError::bad_request)?;
    apply(&ctx.svc, &wanted, &limits).await.map_err(refusal)?;
    Ok(json_reply(200, &body(&ctx.svc)))
}

/// Registers the settings routes.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    r.get("/api/overlord/settings", show);
    r.add("POST", "/api/overlord/settings", small, change);
}
