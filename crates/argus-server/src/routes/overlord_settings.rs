//! The hosted Overlord's settings, for the owner's page only (I-200):
//!
//! - `GET /api/overlord/settings` is `{model, effort, permissionMode, choices: {models: [{id,
//!   label}], efforts, permissionModes: [{id, label}]}}`
//! - `POST /api/overlord/settings` takes any of `{model, effort, permissionMode}`, checks each
//!   against the choices (400 on a bad value), saves it in `config.overlord`, restarts a running
//!   child on the same session and answers the same shape

use argus_core::{ApiError, ErrorCode};
use argus_overlord::session::choices::Change;
use serde_json::Value;

use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::settings::{SettingsFault, apply, body};
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
    let wanted = change_of(&ctx.json().await?)?;
    apply(&ctx.svc, &wanted).await.map_err(refusal)?;
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
