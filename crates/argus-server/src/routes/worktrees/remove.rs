//! Removing one worktree, and pruning the gone ones.

use argus_core::clock::parse_ms;
use argus_worktrees::{RemoveInput, RemoveOutcome, Service};
use serde_json::{Value, to_value};

use super::{caller, id_of, post, reason, serve};
use crate::router::Reply;
use crate::{Ctx, Router};

const REMOVE_BYTES: usize = 8 * 1024;
const PRUNE_BYTES: usize = 4 * 1024;

fn input_of(ctx: &Ctx, key: &str, body: &Value) -> RemoveInput {
    let mut input = RemoveInput::new(key, &id_of(&body["id"]), caller(ctx, body));
    input.branch = body["branch"] == true;
    input.discard = body["discard"] == true;
    input.owner_entry = body["ownerEntry"]
        .as_str()
        .map(|e| e.trim().chars().take(60).collect::<String>())
        .filter(|e| !e.is_empty());
    input.head = body["head"].as_str().map(str::to_owned);
    input.newest_at_ms = body["newestAt"]
        .as_str()
        .and_then(|at| parse_ms(&at.chars().take(40).collect::<String>()));
    input.confirm_young = body["confirmYoung"] == true;
    input.dirty = body["dirty"].as_u64().and_then(|n| u32::try_from(n).ok());
    input.reason = reason(body);
    input
}

fn render(done: RemoveOutcome) -> Value {
    to_value(done).unwrap_or(Value::Null)
}

async fn remove(ctx: Ctx) -> Reply {
    serve(ctx, input_of, Service::remove, render).await
}

async fn prune(ctx: Ctx) -> Reply {
    serve(ctx, input_of, Service::prune, render).await
}

pub(super) fn register(r: &mut Router) {
    post(r, "/api/p/:key/worktrees/remove", REMOVE_BYTES, remove);
    post(r, "/api/p/:key/worktrees/prune", PRUNE_BYTES, prune);
}
