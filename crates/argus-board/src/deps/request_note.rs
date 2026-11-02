//! The words of the inbox message that carries a dependency request to the assignee thread.

use serde_json::Value;

use super::model::Dep;

const SEPARATOR: &str = " | ";

/// The acceptance lines of an item doc, in order.
fn acceptance(item: &Value) -> Vec<&str> {
    item["ac"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|line| line["text"].as_str())
        .collect()
}

/// One line for the inbox of the assignee, parts joined by ` | `:
/// `dep D-11: alpha/I-4 needs your I-9 "Title" | acceptance: a; b | start it: ... or answer: ...`.
/// `doc` is the assignee item as stored; the acceptance part is left out when it has none.
pub(crate) fn request_note(dep: &Dep, item: &str, doc: &Value) -> String {
    let id = &dep.id;
    let title = doc["title"].as_str().unwrap_or(&dep.title);
    let mut parts = vec![format!(
        "dep {id}: {} needs your {item} \"{title}\"",
        dep.requester()
    )];
    let lines = acceptance(doc);
    if !lines.is_empty() {
        parts.push(format!("acceptance: {}", lines.join("; ")));
    }
    parts.push(format!(
        "start it: coord item status {item} doing, or answer: coord dep close {id} --as blocked --reason \"...\""
    ));
    parts.join(SEPARATOR)
}
