//! Removing one worktree, and pruning the gone ones.

use argus_core::clock::parse_ms;
use argus_worktrees::RemoveInput;
use serde_json::{Value, to_value};

use super::{answer, blocking, caller, id_of, reason, service, writable};
use crate::routes::params;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

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

async fn remove(ctx: Ctx) -> Reply {
    let service = service(&ctx)?;
    writable(&ctx)?;
    let key = params::key(&ctx)?;
    let body = ctx.json().await?;
    let input = input_of(&ctx, &key, &body);
    let out = blocking(&ctx, &key, service, move |s| s.remove(&input)).await?;
    answer(out, |done| to_value(done).unwrap_or(Value::Null))
}

async fn prune(ctx: Ctx) -> Reply {
    let service = service(&ctx)?;
    writable(&ctx)?;
    let key = params::key(&ctx)?;
    let body = ctx.json().await?;
    let input = input_of(&ctx, &key, &body);
    let out = blocking(&ctx, &key, service, move |s| s.prune(&input)).await?;
    answer(out, |done| to_value(done).unwrap_or(Value::Null))
}

pub(super) fn register(r: &mut Router) {
    let remove_opts = RouteOpts {
        max_bytes: REMOVE_BYTES,
        ..RouteOpts::default()
    };
    let prune_opts = RouteOpts {
        max_bytes: PRUNE_BYTES,
        ..RouteOpts::default()
    };
    r.add("POST", "/api/p/:key/worktrees/remove", remove_opts, remove);
    r.add("POST", "/api/p/:key/worktrees/prune", prune_opts, prune);
}
