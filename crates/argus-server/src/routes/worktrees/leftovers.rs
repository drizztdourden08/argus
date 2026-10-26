//! Deleting the old leftover folders of a project in the background.

use argus_worktrees::{BatchInput, BatchOutcome};
use serde_json::{Value, json, to_value};

use super::{answer, blocking, caller, id_of, reason, service, writable};
use crate::routes::params;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

const LEFTOVER_BYTES: usize = 64 * 1024;
const IDS_MAX: usize = 500;
const DAYS_MAX: f64 = 3650.0;

fn ids_of(body: &Value) -> Option<Vec<String>> {
    let ids = body["ids"].as_array()?;
    Some(
        ids.iter()
            .filter(|i| i.is_string())
            .map(id_of)
            .filter(|i| !i.is_empty())
            .take(IDS_MAX)
            .collect(),
    )
}

fn input_of(ctx: &Ctx, key: &str, body: &Value) -> BatchInput {
    BatchInput {
        ids: ids_of(body),
        older_than_days: body["olderThanDays"]
            .as_f64()
            .filter(|d| (0.0..=DAYS_MAX).contains(d)),
        confirm_young: body["confirmYoung"] == true,
        reason: reason(body),
        ..BatchInput::new(key, caller(ctx, body))
    }
}

fn skipped(outcome: &BatchOutcome) -> Vec<Value> {
    outcome
        .skipped
        .iter()
        .map(|s| json!({ "id": s.id, "path": s.path, "reason": s.reason }))
        .collect()
}

fn outcome_json(outcome: &BatchOutcome) -> Value {
    json!({
        "started": outcome.started,
        "queued": outcome.started,
        "skipped": skipped(outcome),
        "sizeMB": outcome.size_mb,
        "files": outcome.files,
        "jobs": to_value(&outcome.jobs).unwrap_or(Value::Null),
        "olderThanDays": outcome.older_than_days,
        "text": outcome.text,
    })
}

async fn remove(ctx: Ctx) -> Reply {
    let service = service(&ctx)?;
    writable(&ctx)?;
    let key = params::key(&ctx)?;
    let body = ctx.json().await?;
    let input = input_of(&ctx, &key, &body);
    let out = blocking(&ctx, &key, service, move |s| s.remove_leftovers(&input)).await?;
    answer(out, |done| outcome_json(&done))
}

pub(super) fn register(r: &mut Router) {
    let opts = RouteOpts {
        max_bytes: LEFTOVER_BYTES,
        ..RouteOpts::default()
    };
    r.add("POST", "/api/p/:key/worktrees/leftovers/remove", opts, remove);
}
