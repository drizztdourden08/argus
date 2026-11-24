//! The handover cycle of the accounts service:
//!
//! - `GET /api/accounts/cycle` is `{cycle}`: the current or last `SW-n` with its handovers in
//!   full, `null` before the first
//! - `POST /api/accounts/handover {action, board, thread, ...}` is what `coord handover`
//!   sends for a lead. `action` is `estimate` (`percent`, `minutes`, `agents`, `note`),
//!   `write` (`file`, `agentsDetail: [{id, branch, stop}]`) or `status`. The answer is
//!   `{handover, next, expects, cycle: {id, phase}}`: the lead's record and what Argus expects
//!   of it next, as a word and as a sentence
//! - `action` `resumed` or `not_resumed` (`board`, `thread`) is the resume task's verdict on a
//!   lead after the switch; the answer is `{handover, cycle: {id, phase}}`

use argus_core::lead::canonical;
use std::sync::Arc;

use argus_accounts::handover::{AgentStop, DeskError, Estimate, LeadAction};
use argus_core::clock::now_iso;
use argus_core::{ApiError, ErrorCode};
use serde_json::{Value, json};

use crate::services::accounts_handover::Handovers;
use crate::{Ctx, RouteOpts, Router, reply::json_reply, router::Reply, services::AccountsService};

const HANDOVER_BYTES: usize = 16 * 1024;
const NOTE_MAX: usize = 300;
const FILE_MAX: usize = 400;
const AGENTS_MAX: usize = 50;

fn services(ctx: &Ctx) -> Result<(Arc<AccountsService>, Arc<Handovers>), ApiError> {
    let accounts = ctx
        .svc
        .accounts
        .clone()
        .ok_or_else(|| ApiError::unavailable("the accounts service is not running"))?;
    let handovers = accounts
        .handovers()
        .ok_or_else(|| ApiError::unavailable("the handover desk has not started"))?;
    Ok((accounts, handovers))
}

fn desk_error(e: &DeskError) -> ApiError {
    let code = match e.status() {
        400 => ErrorCode::BadRequest,
        404 => ErrorCode::NotFound,
        409 => ErrorCode::Conflict,
        _ => ErrorCode::Internal,
    };
    ApiError::new(code, e.to_string())
}

fn text<'a>(body: &'a Value, key: &str) -> Option<&'a str> {
    body.get(key)
        .and_then(Value::as_str)
        .filter(|t| !t.trim().is_empty())
}

fn bounded(body: &Value, key: &str, max: usize) -> Result<Option<String>, ApiError> {
    match text(body, key) {
        Some(t) if t.chars().count() > max => Err(ApiError::bad_request(format!(
            "{key} must be at most {max} characters"
        ))),
        other => Ok(other.map(str::to_owned)),
    }
}

fn number(body: &Value, key: &str) -> Result<f64, ApiError> {
    body.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| ApiError::bad_request(format!("{key} must be a number")))
}

fn estimate_of(body: &Value) -> Result<LeadAction, ApiError> {
    let agents = body
        .get("agents")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| ApiError::bad_request("agents must be a whole number"))?;
    Ok(LeadAction::Estimate(Estimate {
        percent: number(body, "percent")?,
        minutes: number(body, "minutes")?,
        agents,
        note: bounded(body, "note", NOTE_MAX)?,
        at: now_iso(),
    }))
}

fn agent_of(raw: &Value) -> Result<AgentStop, ApiError> {
    let field = |key| bounded(raw, key, NOTE_MAX);
    let id = field("id")?.ok_or_else(|| ApiError::bad_request("each agent needs an id"))?;
    Ok(AgentStop {
        id,
        branch: field("branch")?,
        stop: field("stop")?,
    })
}

fn write_of(body: &Value) -> Result<LeadAction, ApiError> {
    let file = bounded(body, "file", FILE_MAX)?
        .ok_or_else(|| ApiError::bad_request("file is required: the handover file you wrote"))?;
    let listed = body
        .get("agentsDetail")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    if listed.len() > AGENTS_MAX {
        return Err(ApiError::bad_request(format!(
            "at most {AGENTS_MAX} agents per handover"
        )));
    }
    let agents = listed.iter().map(agent_of).collect::<Result<_, _>>()?;
    Ok(LeadAction::Write { file, agents })
}

fn action_of(body: &Value) -> Result<LeadAction, ApiError> {
    match text(body, "action") {
        Some("estimate") => estimate_of(body),
        Some("write") => write_of(body),
        Some("status") => Ok(LeadAction::Status),
        _ => Err(ApiError::bad_request(
            "action must be estimate, write or status",
        )),
    }
}

async fn cycle(ctx: Ctx) -> Reply {
    let (_, handovers) = services(&ctx)?;
    let body = json!({ "cycle": handovers.desk().cycle() });
    Ok(json_reply(200, &body))
}

/// `resumed` or `not_resumed`: the resume task's verdict on one lead of the cycle.
fn resume_report(handovers: &Handovers, body: &Value, resumed: bool) -> Reply {
    let board = text(body, "board").ok_or_else(|| ApiError::bad_request("board is required"))?;
    let thread = text(body, "thread").ok_or_else(|| ApiError::bad_request("thread is required"))?;
    let desk = handovers.desk();
    let id = desk
        .cycle()
        .and_then(|c| {
            c.handover_of(board, &canonical(thread))
                .map(|h| h.id.clone())
        })
        .ok_or_else(|| ApiError::not_found("no such lead in the handover cycle"))?;
    let handover = desk
        .mark_resumed(&id, resumed)
        .map_err(|e| desk_error(&e))?;
    let cycle = desk
        .cycle()
        .map(|c| json!({ "id": c.id, "phase": c.phase }));
    Ok(json_reply(
        200,
        &json!({ "handover": handover, "cycle": cycle }),
    ))
}

async fn handover(ctx: Ctx) -> Reply {
    let (accounts, handovers) = services(&ctx)?;
    let body: Value = ctx.json().await?;
    match text(&body, "action") {
        Some("resumed") => return resume_report(&handovers, &body, true),
        Some("not_resumed") => return resume_report(&handovers, &body, false),
        _ => {}
    }
    let action = action_of(&body)?;
    let board = text(&body, "board")
        .ok_or_else(|| ApiError::bad_request("board is required"))?
        .to_owned();
    let thread = text(&body, "thread")
        .ok_or_else(|| ApiError::bad_request("thread is required"))?
        .to_owned();
    let thread = canonical(&thread).into_owned();
    let desk = Arc::clone(&handovers);
    let reply =
        tokio::task::spawn_blocking(move || desk.lead((&board, &thread), action, &accounts))
            .await
            .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?
            .map_err(|e| desk_error(&e))?;
    let cycle = handovers.desk().cycle();
    let body = json!({
        "handover": reply.handover,
        "next": reply.next,
        "expects": reply.expects,
        "cycle": cycle.map(|c| json!({ "id": c.id, "phase": c.phase })),
    });
    Ok(json_reply(200, &body))
}

/// Registers the handover routes.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        max_bytes: HANDOVER_BYTES,
        ..RouteOpts::default()
    };
    r.get("/api/accounts/cycle", cycle);
    r.add("POST", "/api/accounts/handover", opts, handover);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use argus_accounts::handover::LeadAction;
    use serde_json::json;

    use super::action_of;

    #[test]
    fn an_estimate_needs_its_three_numbers() {
        let ok =
            action_of(&json!({"action": "estimate", "percent": 0.6, "minutes": 8, "agents": 3}));
        assert!(matches!(ok, Ok(LeadAction::Estimate(e)) if e.agents == 3));
        assert!(action_of(&json!({"action": "estimate", "percent": 0.6, "minutes": 8})).is_err());
    }

    #[test]
    fn a_write_needs_its_file_and_reads_the_agents() {
        let body = json!({"action": "write", "file": "h.md",
            "agentsDetail": [{"id": "A-2", "branch": "wip/x", "stop": "tests pass"}]});
        let Ok(LeadAction::Write { file, agents }) = action_of(&body) else {
            panic!("not a write");
        };
        assert_eq!(file, "h.md");
        assert_eq!(agents[0].branch.as_deref(), Some("wip/x"));
        assert!(action_of(&json!({"action": "write"})).is_err());
        assert!(
            action_of(&json!({"action": "write", "file": "h.md", "agentsDetail": [{}]})).is_err()
        );
    }

    #[test]
    fn an_unknown_action_is_refused() {
        assert!(action_of(&json!({"action": "switch"})).is_err());
        assert!(matches!(
            action_of(&json!({"action": "status"})),
            Ok(LeadAction::Status)
        ));
    }
}
