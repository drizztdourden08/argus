//! The `require-assignee` rule: the item has an assignee, of one of the kinds named when the
//! rule names any.

use serde_json::Value;

use crate::kanban_config::Rule;

/// The kind of an assignee: `owner`, `agent` (`agent:A-n`), `lead` (`lead:key`) or `thread`.
fn kind_of(assignee: &str) -> &'static str {
    if assignee == "owner" {
        "owner"
    } else if assignee.starts_with("agent:") {
        "agent"
    } else if assignee.starts_with("lead:") {
        "lead"
    } else {
        "thread"
    }
}

fn kind_list(rule: &Rule) -> Vec<&str> {
    rule.params["kinds"]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// The sentence of the break, or `None` when the item has an assignee that fits.
pub(super) fn breach(rule: &Rule, item: &Value) -> Option<String> {
    let kinds = kind_list(rule);
    let kind = item["assignee"]
        .as_str()
        .filter(|a| !a.is_empty())
        .map(kind_of);
    match kind {
        Some(k) if kinds.is_empty() || kinds.contains(&k) => None,
        _ if kinds.is_empty() => Some(format!("{} needs an assignee.", rule.id)),
        _ => Some(format!(
            "{} needs an assignee of kind {}.",
            rule.id,
            kinds.join(" or ")
        )),
    }
}
