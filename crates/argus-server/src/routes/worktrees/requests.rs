//! Asking Argus to remove: the thread's own session may refuse `coord worktree remove`, so it
//! asks and the server's one worker removes, behind the same guards.

use argus_worktrees::{RequestInput, Service};
use serde_json::{Value, json, to_value};

use super::{caller, id_of, post, reason, serve, service};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::params;
use crate::{Ctx, RouteOpts, Router};

const REQUEST_BYTES: usize = 64 * 1024;
const TARGETS_MAX: usize = 1000;
const OWNER_ENTRY_MAX: usize = 60;
const DAYS_MAX: f64 = 3650.0;

fn targets_of(body: &Value) -> Vec<String> {
    body["targets"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter(|t| t.is_string())
                .map(id_of)
                .filter(|t| !t.is_empty())
                .take(TARGETS_MAX)
                .collect()
        })
        .unwrap_or_default()
}

fn owner_entry_of(body: &Value) -> Option<String> {
    body["ownerEntry"]
        .as_str()
        .map(|e| e.trim().chars().take(OWNER_ENTRY_MAX).collect::<String>())
        .filter(|e| !e.is_empty())
}

/// `Number(body.idleMinutes)`: a number, a numeric text, anything else not a number (the request refuses it).
fn idle_of(body: &Value) -> Option<f64> {
    match &body["idleMinutes"] {
        Value::Null => None,
        Value::Number(n) => n.as_f64(),
        Value::String(text) => Some(text.trim().parse().unwrap_or(f64::NAN)),
        _ => Some(f64::NAN),
    }
}

fn input_of(ctx: &Ctx, key: &str, body: &Value) -> RequestInput {
    RequestInput {
        targets: targets_of(body),
        finished: body["finished"] == true,
        all_leftovers: body["allLeftovers"] == true,
        branch: body["branch"] == true,
        discard: body["discard"] == true,
        owner_entry: owner_entry_of(body),
        reason: reason(body),
        older_than_days: body["olderThanDays"]
            .as_f64()
            .filter(|d| (0.0..=DAYS_MAX).contains(d)),
        idle_minutes: idle_of(body),
        ..RequestInput::new(key, caller(ctx, body))
    }
}

async fn make(ctx: Ctx) -> Reply {
    serve(ctx, input_of, Service::request, |queued| {
        to_value(queued).unwrap_or(Value::Null)
    })
    .await
}

async fn list(ctx: Ctx) -> Reply {
    let service = service(&ctx)?;
    let key = params::key(&ctx)?;
    Ok(json_reply(
        200,
        &json!({ "requests": service.request_list(&key) }),
    ))
}

pub(super) fn register(r: &mut Router) {
    let path = "/api/p/:key/worktrees/requests";
    post(r, path, REQUEST_BYTES, make);
    r.add("GET", path, RouteOpts::default(), list);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K1: the targets and the owner entry of a removal request.
    use super::*;

    #[test]
    fn the_ask_takes_strings_trimmed_and_cut_and_an_owner_entry_cut_to_60() {
        let body = json!({ "targets": [" a ", 3, "", "b"], "ownerEntry": format!("  {}  ", "x".repeat(80)) });
        assert_eq!(targets_of(&body), ["a", "b"]);
        assert_eq!(owner_entry_of(&body).map(|e| e.len()), Some(60));
        assert_eq!(owner_entry_of(&json!({ "ownerEntry": "  " })), None);
        assert_eq!(targets_of(&json!({})).len(), 0);
    }
}
