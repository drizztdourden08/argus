//! Notifications to the owner (DESIGN 13.13), a mount of [`Notify`]:
//!
//! - `POST /api/notify {text, title?, level?, project?, thread?, by?}` records a notification and
//!   shows the toast its level asks for: `{recorded, id, n, level, toast, why}`; never refused for
//!   being over a limit; 400 for no text, a text over 240 characters (from the CLI), an unknown
//!   level or a bad project key
//! - `POST /api/notify/test` (the owner's page only) sends a test ping
//! - `GET /api/notify[?limit=&level=&project=]` is `{notifications}`, newest first
//! - `GET /api/notify/settings` is `{os, toast, sound}` per level
//! - `POST /api/notify/settings {toast?, sound?}` (the owner's page only) changes them
//! - `GET/POST /api/notify/auto` is `{os, auto: {handed, ask, assigned, verify, run}}`; the POST
//!   (the owner's page only) takes any subset of those booleans
//!
//! Anyone on this machine may ping. `by` is `thread` unless the caller names itself `overlord`
//! and the board agrees; the page's calls are `page`; `auto` is never taken from outside.

mod auto;
mod ping;
mod settings;

use std::fmt::Display;
use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use argus_notify::{Filter, Level, Notification, Notify, Outcome};
use axum::response::Response;
use serde::Serialize;
use serde_json::{Value, json, to_value};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

const SMALL: usize = 1024;
const PING_BYTES: usize = 8 * 1024;
const DEFAULT_LIMIT: usize = 50;
const LEVEL_WORDS: &str = "info, ping, urgent";

fn service(ctx: &Ctx) -> Result<Arc<Notify>, ApiError> {
    ctx.svc
        .notify
        .clone()
        .ok_or_else(|| ApiError::unavailable("notifications are off"))
}

/// The answer to a settings change: the saved settings, or a 500 that says why they were not.
fn saved_reply<T: Serialize, E: Display>(saved: Result<T, E>) -> Reply {
    let settings = saved.map_err(|e| {
        let why: String = e.to_string().chars().take(120).collect();
        ApiError::new(
            ErrorCode::Internal,
            format!("the settings could not be saved: {why}"),
        )
    })?;
    Ok(json_reply(200, &to_value(settings).unwrap_or(Value::Null)))
}

fn answer(outcome: &Outcome) -> Response {
    let note = outcome.notification.as_ref();
    json_reply(
        200,
        &json!({
            "recorded": outcome.is_recorded(),
            "id": note.map(|n| n.id.clone()),
            "n": note.map(|n| n.n),
            "level": note.map(|n| n.level.as_str()),
            "toast": outcome.toast.as_str(),
            "why": outcome.why,
        }),
    )
}

/// The notification as the Node route lists it.
fn listed(n: &Notification) -> Value {
    json!({
        "id": n.id,
        "n": n.n,
        "at": n.at,
        "level": n.level.as_str(),
        "title": n.title,
        "text": n.text,
        "project": n.project,
        "thread": n.thread,
        "by": to_value(n.by).unwrap_or(Value::Null),
        "toast": n.toast.as_str(),
        "url": n.url,
        "why": n.why,
    })
}

/// `Number(text)` as a whole number: a blank is 0, anything else that is not whole is none.
fn whole(text: &str) -> Option<i64> {
    let text = text.trim();
    if text.is_empty() {
        return Some(0);
    }
    let number: f64 = text.parse().ok()?;
    let whole = number.is_finite() && number.fract().abs() < f64::EPSILON;
    whole.then(|| format!("{number:.0}").parse().ok())?
}

/// The `limit` query: a whole number is itself, anything else is the default.
fn limit_of(text: Option<&str>) -> usize {
    text.and_then(whole).map_or(DEFAULT_LIMIT, |n| {
        usize::try_from(n.max(0)).unwrap_or(usize::MAX)
    })
}

fn filter_of(ctx: &Ctx) -> Result<Filter, ApiError> {
    let level = match ctx.query("level").filter(|l| !l.is_empty()) {
        None => None,
        Some(name) => Some(
            Level::parse(name)
                .ok_or_else(|| ApiError::bad_request(format!("level is {LEVEL_WORDS}")))?,
        ),
    };
    Ok(Filter {
        limit: limit_of(ctx.query("limit")),
        level,
        project: ctx
            .query("project")
            .filter(|p| !p.is_empty())
            .map(str::to_owned),
    })
}

async fn list(ctx: Ctx) -> Reply {
    let notify = service(&ctx)?;
    let filter = filter_of(&ctx)?;
    let found = super::blocking::blocking(move || notify.list(&filter)).await?;
    let notes = found.map_err(|_| ApiError::internal())?;
    let docs: Vec<Value> = notes.iter().map(listed).collect();
    Ok(json_reply(200, &json!({ "notifications": docs })))
}

/// Registers the notification routes.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    let big = RouteOpts {
        max_bytes: PING_BYTES,
        ..RouteOpts::default()
    };
    r.add("POST", "/api/notify", big, ping::send);
    r.add("POST", "/api/notify/test", small, ping::test);
    r.get("/api/notify/settings", settings::read);
    r.add("POST", "/api/notify/settings", small, settings::change);
    r.get("/api/notify/auto", auto::read);
    r.add("POST", "/api/notify/auto", small, auto::change);
    r.get("/api/notify", list);
}
