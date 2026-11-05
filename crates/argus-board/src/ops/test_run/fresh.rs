//! A new run of a plan: an item of type `test-run` that starts running.

use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::runs::RUN;
use super::target::Found;
use crate::Board;
use crate::error::BoardError;
use crate::json::{display, doc_mut, set};
use crate::ops::typed;
use crate::ops::who::Who;
use crate::store::create_doc;

const TITLE_PLAN: usize = 60;

fn asked(found: &Found, n: i64, build: Option<Value>) -> Map<String, Value> {
    let mut fields = Map::new();
    fields.insert("plan".to_owned(), json!(found.plan_id));
    if let Some(build) = build {
        fields.insert("build".to_owned(), build);
    }
    let plan_title = display(&found.plan["title"]);
    let title = format!("Run {n}: {}", js::slice(&plan_title, TITLE_PLAN));
    let made = json!({"title": title, "type": RUN, "stage": "running", "fields": fields});
    made.as_object().cloned().unwrap_or_default()
}

/// Create the run `n` of the plan, running, with the build the op names. Returns its id.
pub(super) fn create(
    board: &mut Board,
    (op, who): (&Value, &Who),
    (found, n, build): (&Found, i64, Option<Value>),
) -> Result<String, BoardError> {
    let owner = display(&found.plan["thread"]);
    let wanted = asked(found, n, build);
    let (fields, _) = typed::create_item(board, (who, op), &wanted, (&owner, "thread"))?;
    let id = create_doc(board, "items", "I", fields, &who.actor_of())?;
    if let Some(run) = doc_mut(board, "items", &id) {
        set(run, "n", n);
        set(run, "startedAt", now_iso());
        set(run, "endedAt", Value::Null);
    }
    Ok(id)
}
