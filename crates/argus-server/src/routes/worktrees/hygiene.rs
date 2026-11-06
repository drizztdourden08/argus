//! The hygiene's two words: a lead's or the Overlord's approval to remove a worktree that holds work, and the
//! Overlord's exception list.

use argus_core::clock::parse_ms;
use argus_worktrees::{KeepInput, RemoveInput, Service};
use serde_json::{Value, to_value};

use super::{caller, id_of, post, reason, serve};
use crate::router::Reply;
use crate::{Ctx, Router};

const BODY_BYTES: usize = 4 * 1024;
const NAME_MAX: usize = 60;
const NOTE_MAX: usize = 300;

fn approve_input(ctx: &Ctx, key: &str, body: &Value) -> RemoveInput {
    let mut input = RemoveInput::new(key, &id_of(&body["id"]), caller(ctx, body));
    input.reason = reason(body);
    input
}

/// `until` as an ISO date or a number of milliseconds.
fn until_of(value: &Value) -> Option<i64> {
    value.as_i64().or_else(|| {
        value
            .as_str()
            .and_then(|at| parse_ms(&at.trim().chars().take(40).collect::<String>()))
    })
}

fn text_of(value: &Value, max: usize) -> Option<String> {
    let text: String = value.as_str()?.trim().chars().take(max).collect();
    (!text.is_empty()).then_some(text)
}

fn keep_input(ctx: &Ctx, key: &str, body: &Value) -> KeepInput {
    KeepInput {
        project: key.to_string(),
        caller: caller(ctx, body),
        id: id_of(&body["id"]),
        note: text_of(&body["note"], NOTE_MAX),
        until_ms: until_of(&body["until"]),
        asked_by: text_of(&body["askedBy"], NAME_MAX),
        clear: body["clear"] == true,
    }
}

/// The approval answers at once with the request it filed: the backup and the removal take minutes.
async fn approve(ctx: Ctx) -> Reply {
    serve(ctx, approve_input, Service::approve, |queued| {
        to_value(queued).unwrap_or(Value::Null)
    })
    .await
}

async fn keep(ctx: Ctx) -> Reply {
    serve(ctx, keep_input, Service::keep, |kept| {
        to_value(kept).unwrap_or(Value::Null)
    })
    .await
}

pub(super) fn register(r: &mut Router) {
    post(r, "/api/p/:key/worktrees/approve", BODY_BYTES, approve);
    post(r, "/api/p/:key/worktrees/keep", BODY_BYTES, keep);
}
