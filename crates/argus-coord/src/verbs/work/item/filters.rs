//! What `item list` and `item tree` narrow by: `--type`, `--assignee` and `--open`.

use argus_board::Board;
use argus_board::registry::registry;
use serde_json::Value;

use crate::verbs::work::run::{Res, Run, bad};

/// The type of an item: its own, else `task`.
pub(super) fn type_of(item: &Value) -> &str {
    item["type"].as_str().unwrap_or("task")
}

/// Who holds the item: its assignee, else its thread.
pub(super) fn assignee_of(item: &Value) -> &str {
    item["assignee"]
        .as_str()
        .or_else(|| item["thread"].as_str())
        .unwrap_or_default()
}

fn type_list() -> String {
    let all: Vec<&str> = registry().types.iter().map(|t| t.id.as_str()).collect();
    all.join(", ")
}

fn only_type(items: &mut Vec<Value>, name: &str) -> Res<()> {
    if registry().item_type(name).is_none() {
        return Err(bad(format!("type must be one of {}.", type_list())));
    }
    items.retain(|i| type_of(i) == name);
    Ok(())
}

fn only_assignee(board: &Board, r: &Run<'_>, items: &mut Vec<Value>, name: &str) -> Res<()> {
    let wanted = if name == "me" {
        r.who(board)?.thread
    } else {
        name.to_owned()
    };
    items.retain(|i| assignee_of(i) == wanted);
    Ok(())
}

/// Whether the call asks for the open items only. A bare `--open` is a flag; `--open I-3` reads
/// the id as its value, which `item tree` takes as the root.
#[must_use]
fn wants_open(r: &Run<'_>) -> bool {
    r.options.get("open").is_some()
}

fn is_open(item: &Value) -> bool {
    !matches!(item["status"].as_str(), Some("done" | "dropped"))
}

/// The items narrowed by the filters of the call.
///
/// # Errors
/// Refused for a type the registry does not know and for `--assignee me` without a caller.
pub(super) fn narrowed(board: &Board, r: &Run<'_>, mut items: Vec<Value>) -> Res<Vec<Value>> {
    if let Some(name) = r.text_opt("type") {
        only_type(&mut items, name)?;
    }
    if let Some(name) = r.text_opt("assignee") {
        only_assignee(board, r, &mut items, name)?;
    }
    if wants_open(r) {
        items.retain(is_open);
    }
    Ok(items)
}
