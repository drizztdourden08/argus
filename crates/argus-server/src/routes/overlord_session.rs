//! The hosted Overlord's session, for the owner's page only:
//! `GET /api/overlord/status` is `{mode, status, pid, startedAt, sessionId}` and
//! `POST /api/overlord/control` with `{action: start|stop|restart}` answers the same shape.

use argus_core::{ApiError, ErrorCode};
use serde_json::{Value, json};

use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::config::is_live;
use crate::services::overlord::control;
use crate::{Ctx, RouteOpts, Router};

const SMALL: usize = 8 * 1024;

fn mode(ctx: &Ctx) -> &'static str {
    if ctx.svc.shadow {
        "shadow"
    } else if is_live(&ctx.svc) {
        "live"
    } else {
        "off"
    }
}

fn status_body(ctx: &Ctx) -> Value {
    let state = ctx.svc.overlord.state();
    json!({
        "mode": mode(ctx),
        "status": state.status,
        "pid": state.pid,
        "startedAt": state.started_at,
        "sessionId": state.session_id,
    })
}

async fn show(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    Ok(json_reply(200, &status_body(&ctx)))
}

fn refusal(fault: control::Refused) -> ApiError {
    ApiError::new(ErrorCode::Conflict, fault.sentence())
}

async fn act(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let body = ctx.json().await?;
    let done = match body["action"].as_str() {
        Some("start") => control::start(&ctx.svc).await,
        Some("stop") => control::stop(&ctx.svc).await,
        Some("restart") => control::restart(&ctx.svc).await,
        _ => {
            return Err(ApiError::bad_request(
                "action must be start, stop or restart",
            ));
        }
    };
    done.map_err(refusal)?;
    Ok(json_reply(200, &status_body(&ctx)))
}

/// Registers the session routes.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    r.get("/api/overlord/status", show);
    r.add("POST", "/api/overlord/control", small, act);
}
