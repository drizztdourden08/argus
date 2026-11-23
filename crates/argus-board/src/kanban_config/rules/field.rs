//! The `require-field` rule: the item has the field set, on the item or among its type fields.

use serde_json::Value;

use crate::kanban_config::Rule;

fn is_set(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(list) => !list.is_empty(),
        _ => true,
    }
}

/// The sentence of the break, or `None` when the field is set.
pub(super) fn breach(rule: &Rule, item: &Value) -> Option<String> {
    let name = rule.params["field"].as_str().unwrap_or_default();
    if is_set(&item[name]) || is_set(&item["fields"][name]) {
        return None;
    }
    Some(format!("{} needs the field {name} set.", rule.id))
}
