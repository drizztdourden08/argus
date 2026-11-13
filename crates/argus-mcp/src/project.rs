//! Which project a call is about, and the shared reading helpers of the docs.

use std::cmp::Ordering;

use argus_core::boards::resolve;
use argus_core::ids::is_key;
use serde_json::{Value, json};

use crate::backend::McpBackend;
use crate::board_names::label;
use crate::error::ToolError;
use crate::json::{display, truthy};
use argus_core::collate;

/// The board id for what a caller typed (an id or an old key), if that board has a board.
#[must_use]
pub fn resolve_project(backend: &dyn McpBackend, key: &str) -> Option<String> {
    let known = backend.project_keys();
    let id = match backend.home() {
        Some(home) => resolve(&home, key),
        None => Some(key.to_owned()),
    };
    id.filter(|id| known.contains(id))
}

/// The board id a call names (a board id or an old key), checked against the boards that exist.
pub fn require_project(backend: &dyn McpBackend, key: Option<&str>) -> Result<String, ToolError> {
    let Some(key) = key.filter(|k| !k.is_empty()) else {
        return Err(ToolError::bad_request(
            "project is required: pass the board id or old key (portfolio_overview lists them).",
        ));
    };
    if !is_key(key) {
        return Err(ToolError::bad_request(format!(
            "\"{key}\" is not a board id or key (lowercase letters, digits, dashes)."
        )));
    }
    resolve_project(backend, key).ok_or_else(|| unknown_board(backend, key))
}

fn unknown_board(backend: &dyn McpBackend, key: &str) -> ToolError {
    let known: Vec<String> = backend
        .project_keys()
        .iter()
        .map(|id| label(backend, id))
        .collect();
    ToolError::not_found(format!("no board for project \"{key}\""))
        .with_details(json!({ "known": known }))
}

/// A doc or a `not_found` saying what is missing.
pub fn need(doc: Option<Value>, label: &str) -> Result<Value, ToolError> {
    doc.ok_or_else(|| ToolError::not_found(format!("no {label}")))
}

/// Neither archived nor deleted.
#[must_use]
pub fn visible(d: &Value) -> bool {
    !truthy(d, "archivedAt") && !truthy(d, "deletedAt")
}

fn id_number(d: &Value) -> f64 {
    let digits: String = display(d, "id")
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    digits.parse().unwrap_or(0.0)
}

/// Order by the number in the id, then by the id as text.
#[must_use]
pub fn by_id(a: &Value, b: &Value) -> Ordering {
    id_number(a)
        .partial_cmp(&id_number(b))
        .unwrap_or(Ordering::Equal)
        .then_with(|| collate::compare(&display(a, "id"), &display(b, "id"), false))
}

/// The first `limit` rows (default 50, at most 500, at least 1).
#[must_use]
pub fn clip<T>(mut list: Vec<T>, limit: Option<u64>) -> Vec<T> {
    list.truncate(usize::try_from(limit.unwrap_or(50).clamp(1, 500)).unwrap_or(50));
    list
}

/// Docs of a collection that pass `keep`, sorted by id.
pub fn sorted_docs(
    backend: &dyn McpBackend,
    key: &str,
    coll: &str,
    keep: impl Fn(&Value) -> bool,
) -> Vec<Value> {
    let mut rows: Vec<Value> = backend
        .docs(key, coll)
        .into_iter()
        .filter(|d| keep(d))
        .collect();
    rows.sort_by(by_id);
    rows
}
