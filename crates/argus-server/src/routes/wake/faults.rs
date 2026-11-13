//! Wake faults: anyone may list them; only the Overlord clears one, and never from a browser.

use argus_core::{ApiError, ErrorCode};
use argus_wake::board::BoardView;
use argus_wake::external::{RequestError, clear_request, no_open_fault};
use argus_wake::service::driver::{SharedWake, lock};
use serde_json::{Value, json, to_value};

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

/// The intake of a call only the Overlord makes: never from a browser (403 with `from_page`), and `parse` checks
/// the body against the Overlord's board. Answers the wake service and the checked request.
pub(crate) async fn overlord_call<R>(
    ctx: &Ctx,
    from_page: &str,
    parse: impl FnOnce(&Value, &str, Option<&BoardView>) -> Result<R, RequestError>,
) -> Result<(SharedWake, R), ApiError> {
    let wake = service(ctx)?;
    if browser_like(ctx) {
        return Err(ApiError::new(ErrorCode::Forbidden, from_page));
    }
    let body = ctx.json().await?;
    let overlord = ctx.svc.overlord();
    let portfolio = ctx.svc.boards().board(&overlord).map(|b| view_of(&b));
    let request = parse(&body, &overlord, portfolio.as_deref()).map_err(|e| refusal(&e))?;
    Ok((wake, request))
}

pub(super) async fn clear(ctx: Ctx) -> Reply {
    let (wake, request) = overlord_call(&ctx, FROM_PAGE, clear_request).await?;
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
