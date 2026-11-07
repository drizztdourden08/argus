//! The hygiene's two words: a lead's or the Overlord's approval to remove a worktree that holds work, and the
//! Overlord's exception list.

use argus_core::clock::parse_ms;
use argus_worktrees::{ApproveInput, KeepInput, RemoveInput, Service, WtError};
use serde_json::{Value, to_value};

use super::{caller, id_of, post, reason, serve};
use crate::router::Reply;
use crate::{Ctx, Router};

const BODY_BYTES: usize = 16 * 1024;
const NAME_MAX: usize = 60;
const NOTE_MAX: usize = 300;

/// An approval: `ids` (several, answered `{queued, refused}`) or the single `id` (answered with the request itself).
struct Approval {
    input: ApproveInput,
    several: bool,
}

fn approve_input(ctx: &Ctx, key: &str, body: &Value) -> Approval {
    let listed = body["ids"].as_array();
    let mut ids: Vec<String> = listed
        .map(|all| all.iter().map(id_of).filter(|id| !id.is_empty()).collect())
        .unwrap_or_default();
    let single = id_of(&body["id"]);
    if listed.is_none() || !single.is_empty() {
        ids.push(single);
    }
    Approval {
        input: ApproveInput {
            project: key.to_string(),
            ids,
            caller: caller(ctx, body),
            reason: reason(body),
        },
        several: listed.is_some(),
    }
}

fn approve_call(service: &Service, approval: &Approval) -> Result<Value, WtError> {
    let input = &approval.input;
    if approval.several {
        return service
            .approve_many(input)
            .map(|done| to_value(done).unwrap_or(Value::Null));
    }
    let mut one = RemoveInput::new(&input.project, &input.ids[0], input.caller.clone());
    one.reason.clone_from(&input.reason);
    service
        .approve(&one)
        .map(|queued| to_value(queued).unwrap_or(Value::Null))
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

/// The approval answers at once with the one request it filed: the backup and the removal take minutes.
async fn approve(ctx: Ctx) -> Reply {
    serve(ctx, approve_input, approve_call, |answer| answer).await
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
