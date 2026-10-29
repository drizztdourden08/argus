//! The `pr-open` op: a new pull request.

use argus_core::js;
use serde_json::{Map, Value, json};

use super::fields;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, section};
use crate::ops::announce::audit;
use crate::ops::result::OpResult;
use crate::ops::target::item_arg;
use crate::ops::who::Who;
use crate::store::create_doc;
use crate::text::clip_text;

fn item_of(board: &Board, value: &Value) -> Result<Value, BoardError> {
    if js::truthy(value) {
        Ok(json!(item_arg(board, value)?))
    } else {
        Ok(Value::Null)
    }
}

fn refuse_twin(board: &Board, head: &str, base: &str) -> Result<(), BoardError> {
    let twin = section(board, "prs")
        .values()
        .find(|p| p["status"] == "open" && p["head"] == head && p["base"] == base);
    match twin {
        Some(p) => Err(refuse(format!(
            "{} is already open for {head} into {base}.",
            display(&p["id"])
        ))),
        None => Ok(()),
    }
}

fn record(board: &Board, set: &Value) -> Result<Map<String, Value>, BoardError> {
    let head = fields::branch("head", &set["head"])?;
    let base = match &set["base"] {
        Value::Null => "main".to_owned(),
        other => fields::branch("base", other)?,
    };
    if head == base {
        return Err(refuse("head and base are the same branch."));
    }
    refuse_twin(board, &head, &base)?;
    let fields = json!({
        "title": fields::title(&set["title"])?,
        "body": fields::body(&set["body"])?,
        "head": head,
        "base": base,
        "headCommit": fields::commit("headCommit", &set["headCommit"])?,
        "status": "open",
        "item": item_of(board, &set["item"])?,
        "mergedAt": null,
        "mergeCommit": null,
        "mergedBy": null,
        "replay": {"state": "pending", "at": null, "url": null},
        "checks": [],
    });
    Ok(match fields {
        Value::Object(m) => m,
        _ => Map::new(),
    })
}

/// Apply a `pr-open` op: the next `PR-n`, open, with its replay block pending.
pub(super) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let fields = record(board, &op["set"])?;
    let title = display(&fields["title"]);
    let id = create_doc(board, "prs", "PR", fields, &who.actor_of())?;
    let said = format!("{} opened {id} ({})", who.label(), clip_text(&title, 50));
    audit(board, &who.actor_of(), &said);
    Ok(OpResult::applied(format!("{id} opened"))
        .with_extra("created", json!({"c": "prs", "id": id})))
}
