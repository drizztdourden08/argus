//! The owner's overrides of the switch process, from the page only (never the coord CLI), and
//! never in shadow mode or a preview:
//!
//! - `POST /api/accounts/switch/skip-step {stage}` skips the step under way: `countdown` ends
//!   the grace minute, `handover` stops waiting for the handovers (the rest are cut off),
//!   `verify` stops waiting for the account check or the sign-in. 409 for `sync` and `switch`,
//!   for a step that is not under way, and when there is nothing to skip; 400 for an unknown word
//! - `POST /api/accounts/switch/skip-thread {board, thread}` skips one lead's wait: ready with
//!   its handover file when it is idle with a current one, else cut off. 404 for an unknown lead,
//!   409 for a closed cycle or a lead that owes nothing
//!
//! Each override is noted in the activity log (kind `accounts-override`) and the step log.

use std::sync::Arc;

use argus_accounts::handover::Cycle;
use argus_core::lead::canonical;
use argus_core::{ApiError, ErrorCode};
use serde_json::{Value, json};

use super::accounts::outward;
use super::accounts_handover::{desk_error, services, text};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::AccountsService;
use crate::services::accounts_handover::{Handovers, use_now};
use crate::services::accounts_override::{
    current_stage, record, stage_named, step_text, thread_text,
};
use crate::services::accounts_rules::skip_countdown;
use crate::services::accounts_switch::{is_running, request_anyway, request_skip_wait};
use crate::services::accounts_switch_log::Stage;
use crate::{AuthKind, Ctx, Router};

const STAGES: &str = "stage must be detect, countdown, handover, sync, switch or verify";

fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::Conflict, message)
}

fn guard(ctx: &Ctx) -> Result<(), ApiError> {
    outward(ctx, "override the switch")?;
    if ctx.auth == Some(AuthKind::Page) {
        return Ok(());
    }
    Err(ApiError::new(
        ErrorCode::Forbidden,
        "only the owner's page may override the switch",
    ))
}

fn join_error(e: &tokio::task::JoinError) -> ApiError {
    ApiError::new(ErrorCode::Internal, e.to_string())
}

/// Ends the wait for the handovers: of the running switch, or of the open cycle alone.
async fn end_handover_wait(
    accounts: &Arc<AccountsService>,
    handovers: &Arc<Handovers>,
) -> Result<bool, ApiError> {
    if is_running() {
        return Ok(request_anyway());
    }
    let (accounts, handovers) = (Arc::clone(accounts), Arc::clone(handovers));
    tokio::task::spawn_blocking(move || {
        let account = handovers.desk().cycle().map(|c| c.account);
        let now_use = account.and_then(|a| use_now(&accounts, &a));
        handovers.cut_off_rest(now_use, &accounts)
    })
    .await
    .map_err(|e| join_error(&e))?
    .map(|_| true)
    .map_err(|e| desk_error(&e))
}

async fn skip_step(ctx: Ctx) -> Reply {
    guard(&ctx)?;
    let (accounts, handovers) = services(&ctx)?;
    let body: Value = ctx.json().await?;
    let stage = text(&body, "stage")
        .and_then(stage_named)
        .ok_or_else(|| ApiError::bad_request(STAGES))?;
    if current_stage(&accounts, &handovers) != Some(stage) {
        return Err(conflict("that step is not the one under way"));
    }
    let ended = match stage {
        Stage::Countdown => skip_countdown(&accounts).await,
        Stage::Handover => end_handover_wait(&accounts, &handovers).await?,
        Stage::Verify => request_skip_wait(),
        Stage::Detect | Stage::Sync | Stage::Switch => {
            return Err(conflict("this step cannot be skipped"));
        }
    };
    if !ended {
        return Err(conflict("there is nothing to skip"));
    }
    let cycle = handovers.desk().cycle().map(|c| c.id);
    let (svc, line) = (ctx.svc.clone(), step_text(stage, cycle.as_deref()));
    tokio::task::spawn_blocking(move || record(&svc, &accounts, stage, &line))
        .await
        .map_err(|e| join_error(&e))?;
    Ok(json_reply(200, &json!({ "skipped": true })))
}

fn answer(cycle: &Cycle, lead: (&str, &str)) -> Value {
    json!({
        "handover": cycle.handover_of(lead.0, lead.1),
        "cycle": { "id": cycle.id, "phase": cycle.phase },
    })
}

async fn skip_thread(ctx: Ctx) -> Reply {
    guard(&ctx)?;
    let (accounts, handovers) = services(&ctx)?;
    let body: Value = ctx.json().await?;
    let board = text(&body, "board")
        .ok_or_else(|| ApiError::bad_request("board is required"))?
        .to_owned();
    let thread = text(&body, "thread")
        .ok_or_else(|| ApiError::bad_request("thread is required"))?;
    let thread = canonical(thread).into_owned();
    let svc = ctx.svc.clone();
    let done = tokio::task::spawn_blocking(move || {
        let lead = (board.as_str(), thread.as_str());
        let file = handovers.handed_over(lead);
        let out = handovers
            .skip_lead(lead, file.as_deref(), &accounts)
            .map_err(|e| desk_error(&e))?;
        let ids = out.cycle.handover_of(lead.0, lead.1).map(|h| h.id.clone());
        let line = thread_text(
            lead,
            (ids.as_deref().unwrap_or("?"), &out.cycle.id),
            file.is_some(),
        );
        record(&svc, &accounts, Stage::Handover, &line);
        Ok::<_, ApiError>(answer(&out.cycle, lead))
    })
    .await
    .map_err(|e| join_error(&e))??;
    Ok(json_reply(200, &done))
}

/// Registers the override routes.
pub fn register(r: &mut Router) {
    r.post("/api/accounts/switch/skip-step", skip_step);
    r.post("/api/accounts/switch/skip-thread", skip_thread);
}
