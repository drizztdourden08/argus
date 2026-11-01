//! The process monitor (DESIGN 13.10), a mount of [`ProcsService`]:
//!
//! - `GET /api/procs[?rule=r]` is `{meta, procs}`, the last sample's listing, the biggest first
//! - `POST /api/procs/sample` samples now and answers `{meta}`
//! - `GET /api/procs/rules` is `{rules, file, error, protectedNames}`
//! - `POST /api/procs/rules {rule, by?}` creates or changes a rule: `{rule, created}`
//! - `POST /api/procs/rules/remove {name, by?}` is `{removed: true, name}`, 404 for no such rule
//! - `POST /api/procs/kill {pid, startedAt, tree?, reason?, by?}` ends one listed process
//! - `POST /api/procs/kill-rule {rule, olderThanMin?, idleOnly?, reason?, by?}` ends a rule's
//! - `POST /api/p/:key/procs/:pid/kill {by, tree?, reason?}` is a thread's kill of its own
//!   process ([`own`])
//!
//! Who may change rules or kill: the owner's page, or the Overlord (the coordinator thread of
//! the portfolio board) named in `by` from the CLI. Anyone may read. A thread may kill only its
//! own project's processes, through the route of its project.

mod own;

use std::sync::Arc;

use argus_board::names::MAIN_KEY;
use argus_core::clock::iso_from_ms;
use argus_core::{ApiError, ErrorCode};
use argus_procs::{
    ErrorCode as ProcsCode, KillRequest, KillRuleRequest, ProcsError, ProcsService, Source,
};
use argus_store::{ActivityLevel, NewActivity};
use axum::response::Response;
use serde::Serialize;
use serde_json::{Value, json, to_value};

use super::blocking::blocking;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{AuthKind, Ctx, RouteOpts, Router};

const REASON_MAX: usize = 200;
const SMALL: usize = 4 * 1024;
const RULE_BYTES: usize = 16 * 1024;
const ENDED: [&str; 2] = ["done", "archived"];
const RULE_KIND: &str = "process-rule";
const KILL_KIND: &str = "process-kill";

fn service(ctx: &Ctx) -> Result<Arc<ProcsService>, ApiError> {
    ctx.svc
        .procs
        .clone()
        .ok_or_else(|| ApiError::unavailable("the process monitor is not running"))
}

fn refusal(e: &ProcsError) -> ApiError {
    let code = match e.code() {
        ProcsCode::BadRequest => ErrorCode::BadRequest,
        ProcsCode::NotFound => ErrorCode::NotFound,
        ProcsCode::Forbidden => ErrorCode::Forbidden,
        ProcsCode::Conflict => ErrorCode::Conflict,
        ProcsCode::Internal => ErrorCode::Internal,
    };
    ApiError::new(code, e.message())
}

fn answer(body: &impl Serialize) -> Response {
    json_reply(200, &to_value(body).unwrap_or(Value::Null))
}

/// Who asked: the page, or the CLI as the Overlord's thread `actor`.
struct Caller {
    source: Source,
}

impl Caller {
    fn words(&self) -> String {
        match &self.source {
            Source::Cli { actor: Some(a) } => format!("by {a}"),
            _ => "from the page".to_owned(),
        }
    }
}

fn is_overlord(ctx: &Ctx, by: &str) -> bool {
    let board = ctx.svc.boards().board(MAIN_KEY);
    let thread = board.as_ref().map(|b| &b["threads"][by]);
    thread.is_some_and(|t| {
        t["role"] == "coordinator" && !ENDED.contains(&t["status"].as_str().unwrap_or(""))
    })
}

fn caller(ctx: &Ctx, body: &Value, what: &str) -> Result<Caller, ApiError> {
    if ctx.auth == Some(AuthKind::Page) {
        return Ok(Caller {
            source: Source::Page,
        });
    }
    let by = body["by"].as_str().unwrap_or("");
    if is_overlord(ctx, by) {
        return Ok(Caller {
            source: Source::Cli {
                actor: Some(by.to_owned()),
            },
        });
    }
    let named: String = by.chars().take(40).collect();
    let who = if named.is_empty() {
        String::new()
    } else {
        format!(" ({named})")
    };
    let text = format!(
        "refused to {what} from the CLI{who}: only the Overlord or the owner's page may do it"
    );
    ctx.svc
        .activity
        .add(NewActivity::new(KILL_KIND, ActivityLevel::Warn, text));
    Err(ApiError::new(
        ErrorCode::Forbidden,
        format!(
            "only the Overlord (the coordinator thread of project {MAIN_KEY}) or the owner's page may {what}"
        ),
    ))
}

fn reason_of(body: &Value) -> Option<String> {
    let text = body["reason"].as_str()?;
    let folded = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = folded.chars().take(REASON_MAX).collect();
    (!cut.is_empty()).then_some(cut)
}

fn pid_of(body: &Value) -> u32 {
    let n = match &body["pid"] {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    };
    n.and_then(|n| u32::try_from(n).ok()).unwrap_or(0)
}

fn started_at_of(body: &Value) -> Option<String> {
    match &body["startedAt"] {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => n.as_i64().map(iso_from_ms),
        _ => None,
    }
}

fn older_than_of(body: &Value) -> Result<Option<f64>, ApiError> {
    let bad = || ApiError::bad_request("olderThanMin must be a number of minutes");
    let minutes = match &body["olderThanMin"] {
        Value::Null => return Ok(None),
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    };
    minutes
        .filter(|m| m.is_finite() && *m >= 0.0)
        .map(Some)
        .ok_or_else(bad)
}

async fn listing(ctx: Ctx) -> Reply {
    let procs = service(&ctx)?;
    let docs = procs.list(ctx.query("rule").filter(|r| !r.is_empty()));
    Ok(answer(&json!({ "meta": procs.meta(), "procs": docs })))
}

async fn sample(ctx: Ctx) -> Reply {
    let procs = service(&ctx)?;
    let meta = blocking(move || procs.sample_now()).await?;
    Ok(answer(&json!({ "meta": meta })))
}

async fn rules(ctx: Ctx) -> Reply {
    Ok(answer(&service(&ctx)?.rules()))
}

async fn set_rule(ctx: Ctx) -> Reply {
    let procs = service(&ctx)?;
    let body = ctx.json().await?;
    if !body["rule"].is_object() {
        return Err(ApiError::bad_request("send {rule: {name, ...}}"));
    }
    let who = caller(&ctx, &body, "change the process rules")?;
    let change = procs.set_rule(&body["rule"]).map_err(|e| refusal(&e))?;
    let verb = if change.created { "added" } else { "changed" };
    let text = format!("rule {} {verb} {}", change.rule.name, who.words());
    ctx.svc
        .activity
        .add(NewActivity::new(RULE_KIND, ActivityLevel::Info, text));
    Ok(answer(
        &json!({ "rule": change.rule, "created": change.created }),
    ))
}

async fn remove_rule(ctx: Ctx) -> Reply {
    let procs = service(&ctx)?;
    let body = ctx.json().await?;
    let name = body["name"].as_str().unwrap_or("").to_owned();
    let who = caller(&ctx, &body, "change the process rules")?;
    if !procs.remove_rule(&name).map_err(|e| refusal(&e))? {
        return Err(ApiError::not_found(format!("no rule \"{name}\"")));
    }
    let text = format!("rule {name} removed {}", who.words());
    ctx.svc
        .activity
        .add(NewActivity::new(RULE_KIND, ActivityLevel::Info, text));
    Ok(answer(&json!({ "removed": true, "name": name })))
}

async fn kill(ctx: Ctx) -> Reply {
    let procs = service(&ctx)?;
    let body = ctx.json().await?;
    let who = caller(&ctx, &body, "kill a process")?;
    let request = KillRequest {
        pid: pid_of(&body),
        started_at: started_at_of(&body),
        tree: body["tree"] == json!(true),
        reason: reason_of(&body),
        source: who.source,
    };
    let done = blocking(move || procs.kill(&request)).await?;
    Ok(answer(&done.map_err(|e| refusal(&e))?))
}

async fn kill_rule(ctx: Ctx) -> Reply {
    let procs = service(&ctx)?;
    let body = ctx.json().await?;
    let Some(rule) = body["rule"].as_str().filter(|r| !r.is_empty()) else {
        return Err(ApiError::bad_request("rule is required"));
    };
    let older_than_min = older_than_of(&body)?;
    let who = caller(&ctx, &body, "kill processes")?;
    let request = KillRuleRequest {
        rule: rule.to_owned(),
        older_than_min,
        idle_only: body["idleOnly"] == json!(true),
        reason: reason_of(&body),
        source: who.source,
    };
    let done = blocking(move || procs.kill_rule(&request)).await?;
    Ok(answer(&done.map_err(|e| refusal(&e))?))
}

/// Registers the process monitor routes.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    let rule = RouteOpts {
        max_bytes: RULE_BYTES,
        ..RouteOpts::default()
    };
    r.get("/api/procs", listing);
    r.post("/api/procs/sample", sample);
    r.get("/api/procs/rules", rules);
    r.add("POST", "/api/procs/rules", rule, set_rule);
    r.add("POST", "/api/procs/rules/remove", small, remove_rule);
    r.add("POST", "/api/procs/kill", small, kill);
    r.add("POST", "/api/procs/kill-rule", small, kill_rule);
    own::register(r, small);
}
