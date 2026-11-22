//! The hosted Overlord's session, for the owner's page only:
//! `GET /api/overlord/status` is `{mode, status, activity, pid, startedAt, sessionId}` (`activity` is
//! `off`, `idle`, `thinking` or `working`) with `nextRuleAt`, `nextRuleEveryMin` and `nextRuleTitle`
//! (the enabled rule due first, all null with none or when the rules cannot be read) and
//! `POST /api/overlord/control` with `{action: start|stop|restart}` answers the same shape.

use argus_core::{ApiError, ErrorCode};
use argus_overlord::rules::next_rule;
use argus_overlord::session::activity::activity;
use serde_json::{Value, json};

use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::config::{is_live, is_shadow};
use crate::services::overlord::control;
use crate::services::overlord::rules;
use crate::{Ctx, RouteOpts, Router};

const SMALL: usize = 8 * 1024;

fn mode(ctx: &Ctx) -> &'static str {
    if ctx.svc.shadow {
        "shadow"
    } else if is_live(&ctx.svc) {
        "live"
    } else if is_shadow(&ctx.svc) {
        "shadow"
    } else {
        "off"
    }
}

/// The next rule's time, period and title; all null with none or when the store fails.
fn next_rule_fields(ctx: &Ctx) -> (Value, Value, Value) {
    let all = rules::list(&ctx.svc).unwrap_or_default();
    match next_rule(&all) {
        Some(rule) => (
            json!(rule.next_run_at),
            json!(rule.every_min),
            json!(rule.title),
        ),
        None => (Value::Null, Value::Null, Value::Null),
    }
}

fn status_body(ctx: &Ctx) -> Value {
    let (rule_at, rule_every, rule_title) = next_rule_fields(ctx);
    let state = ctx.svc.overlord.state();
    let tool = ctx.svc.overlord.link().is_some_and(|l| l.tool_in_flight());
    json!({
        "mode": mode(ctx),
        "status": state.status,
        "activity": activity(state.status, tool),
        "pid": state.pid,
        "startedAt": state.started_at,
        "sessionId": state.session_id,
        "nextRuleAt": rule_at,
        "nextRuleEveryMin": rule_every,
        "nextRuleTitle": rule_title,
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
