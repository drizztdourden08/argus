//! What an agent record is linked to (R21, R5): the items it works on (`items`, with `item` kept as
//! the first one for every reader of the older field), the place it runs (`worktree`, or `master`
//! for the main checkout) and the records it waits on (`blockedBy`). Old records carry only `item`
//! and `worktree`; every reader here falls back to them.

use serde_json::{Value, json};

use crate::json::{set, truthy};

/// A text field that holds something.
fn filled<'a>(agent: &'a Value, key: &str) -> Option<&'a str> {
    agent[key].as_str().map(str::trim).filter(|s| !s.is_empty())
}

fn strings(agent: &Value, key: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let listed = agent[key].as_array().into_iter().flatten();
    for id in listed.filter_map(|v| v.as_str()).map(str::trim) {
        if !id.is_empty() && !out.iter().any(|x| x == id) {
            out.push(id.to_owned());
        }
    }
    out
}

/// The items an agent works on: `items`, else the single `item` of an older record.
#[must_use]
pub fn items_of(agent: &Value) -> Vec<String> {
    let listed = strings(agent, "items");
    if !listed.is_empty() {
        return listed;
    }
    filled(agent, "item")
        .map(str::to_owned)
        .into_iter()
        .collect()
}

/// The ids an agent waits on, empty when it has none.
#[must_use]
pub fn blocked_by_of(agent: &Value) -> Vec<String> {
    strings(agent, "blockedBy")
}

/// Where an agent runs and how Argus knows. `master` or a `worktree` the lead named is `manual`; with
/// neither, the folder `agent-<claudeId>` Claude makes for it is `auto`; an agent with no Claude id yet
/// has no place (`null`, `null`).
#[must_use]
pub fn where_of(agent: &Value) -> (Value, Value) {
    if truthy(agent, "master") {
        return (json!("master"), json!("manual"));
    }
    if let Some(worktree) = filled(agent, "worktree") {
        return (json!(worktree), json!("manual"));
    }
    let claude = filled(agent, "claudeId").or_else(|| filled(agent, "hookId"));
    match claude {
        Some(id) => (json!(format!("agent-{id}")), json!("auto")),
        None => (Value::Null, Value::Null),
    }
}

/// Keeps `item` and `items` one fact after an edit of either, and `master` and `worktree` from both
/// being set: the field the edit touched wins.
pub fn settle(agent: &mut Value, edited: &[String]) {
    let touched = |key: &str| edited.iter().any(|f| f == key);
    if touched("items") && !touched("item") {
        let first = items_of(agent).into_iter().next();
        set(agent, "item", first.map_or(Value::Null, Value::from));
    } else if touched("item") && !touched("items") {
        let own = filled(agent, "item").map(str::to_owned);
        set(agent, "items", json!(own.into_iter().collect::<Vec<_>>()));
    }
    if touched("master") && truthy(agent, "master") && !touched("worktree") {
        set(agent, "worktree", Value::Null);
    } else if touched("worktree") && filled(agent, "worktree").is_some() && !touched("master") {
        set(agent, "master", false);
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_old_record_reads_its_item_as_the_list() {
        assert_eq!(items_of(&json!({"item": "I-4"})), ["I-4"]);
        assert_eq!(items_of(&json!({"item": null})).len(), 0);
        assert_eq!(
            items_of(&json!({"items": ["I-2", "I-2", "I-9"], "item": "I-2"})),
            ["I-2", "I-9"]
        );
    }

    #[test]
    fn where_is_manual_for_master_or_a_named_worktree_and_auto_from_the_claude_id() {
        assert_eq!(
            where_of(&json!({"master": true, "worktree": "x"})),
            (json!("master"), json!("manual"))
        );
        assert_eq!(
            where_of(&json!({"worktree": "X:/p/.claude/worktrees/agent-a1"})),
            (json!("X:/p/.claude/worktrees/agent-a1"), json!("manual"))
        );
        assert_eq!(
            where_of(&json!({"claudeId": "a1b2"})),
            (json!("agent-a1b2"), json!("auto"))
        );
        assert_eq!(
            where_of(&json!({"hookId": "c3"})),
            (json!("agent-c3"), json!("auto"))
        );
        assert_eq!(
            where_of(&json!({"worktree": null})),
            (Value::Null, Value::Null)
        );
    }

    #[test]
    fn an_edit_of_items_or_item_moves_the_other_and_master_clears_the_worktree() {
        let mut agent = json!({"item": "I-1", "items": ["I-1"], "worktree": "w", "master": false});
        agent["items"] = json!(["I-5", "I-6"]);
        settle(&mut agent, &["items".to_owned()]);
        assert_eq!(agent["item"], "I-5");
        agent["item"] = json!("I-7");
        settle(&mut agent, &["item".to_owned()]);
        assert_eq!(agent["items"], json!(["I-7"]));
        agent["master"] = json!(true);
        settle(&mut agent, &["master".to_owned()]);
        assert_eq!(agent["worktree"], Value::Null);
        agent["worktree"] = json!("w2");
        settle(&mut agent, &["worktree".to_owned()]);
        assert_eq!(agent["master"], false);
    }
}
