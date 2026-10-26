//! Wake faults: anyone may list them; only the Overlord clears one, and never from a browser.

use argus_core::{ApiError, ErrorCode};
use argus_wake::external::{clear_request, no_open_fault};
use argus_wake::reasons::MAIN_SCOPE;
use argus_wake::service::driver::lock;
use serde_json::{json, to_value};

use super::{refusal, service};
use crate::adapters::wake::view_of;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::blocking::blocking;
use crate::{AuthKind, Ctx};

const FROM_PAGE: &str = "a wake fault cannot be cleared from the page: only the Overlord clears it, with `coord wake-fault clear`";

pub(super) async fn list(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let all = matches!(ctx.query("all"), Some("1" | "true"));
    let faults = lock(&wake).faults(all);
    Ok(json_reply(
        200,
        &json!({ "faults": to_value(faults).unwrap_or_default() }),
    ))
}

/// A request a browser sent: from the page, or with fetch metadata or an Origin.
fn browser_like(ctx: &Ctx) -> bool {
    ctx.auth == Some(AuthKind::Page)
        || ctx.header("sec-fetch-site").is_some()
        || ctx.header("origin").is_some()
}

pub(super) async fn clear(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    if browser_like(&ctx) {
        return Err(ApiError::new(ErrorCode::Forbidden, FROM_PAGE));
    }
    let body = ctx.json().await?;
    let portfolio = ctx.svc.boards().board(MAIN_SCOPE).map(|b| view_of(&b));
    let request = clear_request(&body, portfolio.as_deref()).map_err(|e| refusal(&e))?;
    let (project, thread) = (request.project.clone(), request.thread.clone());
    let cleared = blocking(move || {
        lock(&wake).clear_fault(
            &request.project,
            &request.thread,
            &request.by,
            &request.note,
        )
    })
    .await?;
    let fault = cleared.ok_or_else(|| refusal(&no_open_fault(&project, &thread)))?;
    Ok(json_reply(
        200,
        &json!({ "cleared": true, "fault": to_value(fault).unwrap_or_default() }),
    ))
}
