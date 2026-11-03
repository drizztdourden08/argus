//! What the write routes of the local remote share: the shadow refusal, the JSON object body and
//! the name of who acted.

use argus_core::{ApiError, ErrorCode};
use serde_json::Value;

use crate::{AuthKind, Ctx};

const BY_MAX: usize = 60;

/// Refuses a board write in shadow mode, where Argus writes no board. A preview writes boards.
///
/// # Errors
/// 403 `read_only` in shadow mode.
pub fn writable(ctx: &Ctx) -> Result<(), ApiError> {
    if !ctx.svc.board_writes {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus does not write boards or start runs",
        ));
    }
    Ok(())
}

/// Refuses an outward effect (a run, a check run, a process kill) in shadow mode and in a preview.
///
/// # Errors
/// 403 `read_only` in shadow mode and in a preview.
pub fn outward(ctx: &Ctx) -> Result<(), ApiError> {
    if ctx.svc.shadow {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow or preview mode: Argus has no outward effect (no run, no kill)",
        ));
    }
    Ok(())
}

/// The JSON body of a write, which must be an object.
///
/// # Errors
/// As [`Ctx::json`], and 400 when the body is not an object.
pub async fn object_body(ctx: &Ctx) -> Result<Value, ApiError> {
    let body = ctx.json().await?;
    if body.is_object() {
        Ok(body)
    } else {
        Err(ApiError::bad_request("expected a JSON object"))
    }
}

/// Who acted: `human` for the owner's page, else the `by` the caller named (60 characters at
/// most), else `cli`.
#[must_use]
pub fn actor(ctx: &Ctx, body: &Value) -> String {
    if ctx.auth == Some(AuthKind::Page) {
        return "human".to_owned();
    }
    let named = body["by"].as_str().map(str::trim).filter(|b| !b.is_empty());
    named.map_or_else(|| "cli".to_owned(), |b| b.chars().take(BY_MAX).collect())
}
