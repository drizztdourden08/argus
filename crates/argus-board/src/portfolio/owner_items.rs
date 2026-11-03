//! What is assigned to the owner on a project: the open items that name him as `assignee` and the
//! pending to-dos that wait for him. The card shows the count and the Notifications tab lists them.

use serde_json::{Value, json};

use crate::json::display;

/// The stage an item names, else its lane.
/// Does this to-do wait for the owner? One with no assignee is his, as every to-do was before assignees;
/// one assigned to a thread or a lead waits for them (the rule of `refresh_needs_human`).
pub(super) fn waits_on_owner(todo: &Value) -> bool {
    matches!(todo["assignee"].as_str(), None | Some("owner"))
}

fn stage_of(item: &Value) -> Value {
    item["stage"]
        .as_str()
        .filter(|stage| !stage.is_empty())
        .or_else(|| item["status"].as_str())
        .map_or_else(|| json!("todo"), |stage| json!(stage))
}

fn item_row(item: &Value) -> Value {
    json!({
        "id": item["id"], "title": item["title"],
        "type": item["type"].as_str().unwrap_or("task"), "stage": stage_of(item),
    })
}

fn todo_row(todo: &Value) -> Value {
    json!({"id": todo["id"], "title": display(&todo["text"]), "type": "todo", "stage": "pending"})
}

/// The items and the to-dos that wait for the owner: `[{id, title, type, stage}]`, items first.
pub(super) fn owner_items(items: &[&Value], actions: &[&Value]) -> Vec<Value> {
    let open = |i: &&&Value| {
        i["assignee"] == "owner" && !matches!(i["status"].as_str(), Some("done" | "dropped"))
    };
    let pending = |a: &&&Value| a["status"] == "pending" && waits_on_owner(a);
    let items = items.iter().filter(open).map(|i| item_row(i));
    let todos = actions.iter().filter(pending).map(|a| todo_row(a));
    items.chain(todos).collect()
}
