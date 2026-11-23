//! The `require-parent` rule: the item has a parent of one of the types named.

use serde_json::Value;

use crate::Board;
use crate::json::doc;
use crate::kanban_config::Rule;

const DEFAULT_TYPE: &str = "task";

fn type_list(rule: &Rule) -> Vec<&str> {
    rule.params["types"]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// The sentence of the break, or `None` when the item has a parent of an allowed type.
pub(super) fn breach(board: &Board, rule: &Rule, item: &Value) -> Option<String> {
    let types = type_list(rule);
    let parent_type = item["parent"]
        .as_str()
        .and_then(|id| doc(board, "items", id))
        .map(|p| p["type"].as_str().unwrap_or(DEFAULT_TYPE));
    match parent_type {
        Some(ty) if types.contains(&ty) => None,
        _ => Some(format!(
            "{} needs a parent of type {}.",
            rule.id,
            types.join(" or ")
        )),
    }
}
