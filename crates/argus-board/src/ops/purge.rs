//! The `purge` op: remove records for good, after a restorable backup. Only the coordinator and
//! the owner's page may do it; the owner confirms with the project key against the counts they
//! looked at.

use std::path::Path;

use argus_core::js;
use serde_json::{Value, json};

use super::authorize::{Check, authorize};
use super::backup::{apply_purge, count_line};
use super::plan::purge_plan;
use super::result::OpResult;
use super::who::Who;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, obj, strict_eq, to_number};

const STALE: &str = "board changed, review again";

/// The criteria with the calling thread protected (the owner protects none).
fn criteria_for(op: &Value, who: &Who) -> Value {
    let mut criteria = match &op["criteria"] {
        Value::Object(c) => c.clone(),
        _ => serde_json::Map::new(),
    };
    let protect: Vec<&str> = if who.is_human() {
        Vec::new()
    } else {
        who.self_id().into_iter().collect()
    };
    criteria.insert("protect".into(), json!(protect));
    Value::Object(criteria)
}

/// Apply a `purge` op.
pub(crate) fn handle(
    board: &mut Board,
    op: &Value,
    who: &Who,
    home: Option<&Path>,
) -> Result<OpResult, BoardError> {
    authorize(who, &Check::new("purge", "threads", None))?;
    if !strict_eq(&op["confirm"], &board["project"]) {
        return Err(refuse("the confirm text must equal the project key."));
    }
    if who.is_human() && !op["expect"].is_object() {
        return Err(refuse(STALE));
    }
    let plan = purge_plan(board, &criteria_for(op, who))?;
    for (key, max) in obj(&op["expect"]) {
        let planned = plan.counts.get(key).map_or(0.0, |n| to_number(Some(n)));
        if planned > to_number(Some(max)) {
            return Err(refuse(STALE));
        }
    }
    if !plan.counts.values().any(js::truthy) {
        return Err(refuse("nothing to purge."));
    }
    let Some(home) = home else {
        return Err(refuse(format!(
            "purging {} needs the coord home.",
            display(&board["project"])
        )));
    };
    let done = apply_purge(home, board, &plan, who)?;
    Ok(OpResult::applied(format!(
        "purged {}; backup {}",
        count_line(&done.counts),
        done.file
    ))
    .with_extra("file", json!(done.file))
    .with_extra("counts", Value::Object(done.counts)))
}
