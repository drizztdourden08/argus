//! `GET /api/activity[?limit=&kind=&level=]` (DESIGN 13.11): `{activity}`, newest first.

use argus_core::ApiError;
use argus_store::{ActivityFilter, ActivityLevel};
use serde_json::json;

use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

const DEFAULT_LIMIT: i64 = 100;

/// The `limit` query: absent is 100, blank is 0 (then 1), a whole number is itself, anything
/// else is 100. The log clamps it to 1..1000.
fn limit_of(text: Option<&str>) -> i64 {
    match text.map(str::trim) {
        None => DEFAULT_LIMIT,
        Some("") => 0,
        Some(t) => t.parse().unwrap_or(DEFAULT_LIMIT),
    }
}

fn level_of(text: Option<&str>) -> Result<Option<ActivityLevel>, ApiError> {
    match text.filter(|l| !l.is_empty()) {
        None => Ok(None),
        Some(name) => ActivityLevel::parse(name)
            .map(Some)
            .ok_or_else(|| ApiError::bad_request("level is info, warn or error")),
    }
}

async fn list(ctx: Ctx) -> Reply {
    let filter = ActivityFilter {
        limit: limit_of(ctx.query("limit")),
        kind: ctx
            .query("kind")
            .filter(|k| !k.is_empty())
            .map(str::to_owned),
        level: level_of(ctx.query("level"))?,
    };
    let rows = ctx.svc.activity.list(&filter)?;
    let docs: Vec<_> = rows
        .iter()
        .map(argus_store::ActivityEntry::to_doc)
        .collect();
    Ok(json_reply(200, &json!({ "activity": docs })))
}

/// Registers the activity route.
pub fn register(r: &mut Router) {
    r.get("/api/activity", list);
}
