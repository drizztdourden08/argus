//! The timed rules of the hosted Overlord, for the owner's page only:
//!
//! - `GET /api/overlord/rules` is `{rules, nextDueAt}`
//! - `POST /api/overlord/rules` with `{title, prompt, everyMin, enabled?}` adds an owner rule
//! - `PUT /api/overlord/rules/<id>` takes any of those fields and answers the rule
//! - `DELETE /api/overlord/rules/<id>` answers `{ok: true}`
//! - `POST /api/overlord/rules/<id>/run` delivers the rule now; 409 when no child runs
//!
//! Bad input is 400 with the sentence, an unknown id is 404. Writes need the strict origin check.

use argus_core::{ApiError, ErrorCode};
use argus_overlord::rules::{By, next_due_at, parse_input, parse_patch};
use serde_json::json;

use super::blocking::blocking;
use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::rules::{self, RuleFault};
use crate::{Ctx, RouteOpts, Router};

/// 4000 characters at four bytes each, and the JSON around them.
const BODY_MAX: usize = 32 * 1024;

fn refusal(fault: RuleFault) -> ApiError {
    match fault {
        RuleFault::Invalid(text) => ApiError::bad_request(text),
        RuleFault::Unknown => ApiError::new(ErrorCode::NotFound, "no such rule"),
        RuleFault::NotRunning => {
            ApiError::new(ErrorCode::Conflict, "the hosted Overlord is not running")
        }
        RuleFault::Store => ApiError::new(ErrorCode::Internal, "the rules could not be read"),
    }
}

async fn list(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let svc = ctx.svc.clone();
    let all = blocking(move || rules::list(&svc))
        .await?
        .map_err(refusal)?;
    let next = next_due_at(&all);
    Ok(json_reply(200, &json!({ "rules": all, "nextDueAt": next })))
}

async fn add(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let input = parse_input(&ctx.json().await?).map_err(ApiError::bad_request)?;
    let svc = ctx.svc.clone();
    let made = blocking(move || rules::add(&svc, &input, By::Owner))
        .await?
        .map_err(refusal)?;
    Ok(json_reply(200, &json!(made)))
}

async fn change(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let patch = parse_patch(&ctx.json().await?).map_err(ApiError::bad_request)?;
    let svc = ctx.svc.clone();
    let id = ctx.param("id").to_owned();
    let saved = blocking(move || rules::update(&svc, &id, &patch))
        .await?
        .map_err(refusal)?;
    Ok(json_reply(200, &json!(saved)))
}

async fn remove(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let svc = ctx.svc.clone();
    let id = ctx.param("id").to_owned();
    blocking(move || rules::remove(&svc, &id))
        .await?
        .map_err(refusal)?;
    Ok(json_reply(200, &json!({ "ok": true })))
}

async fn run(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let svc = ctx.svc.clone();
    let id = ctx.param("id").to_owned();
    let ran = blocking(move || rules::run_now(&svc, &id))
        .await?
        .map_err(refusal)?;
    Ok(json_reply(200, &json!(ran)))
}

/// Registers the rule routes.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        max_bytes: BODY_MAX,
        ..RouteOpts::default()
    };
    r.get("/api/overlord/rules", list);
    r.add("POST", "/api/overlord/rules", opts, add);
    r.add("PUT", "/api/overlord/rules/:id", opts, change);
    r.add("DELETE", "/api/overlord/rules/:id", opts, remove);
    r.add("POST", "/api/overlord/rules/:id/run", opts, run);
}
