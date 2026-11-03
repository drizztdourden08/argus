//! The line an item verb adds when moving an item closed or brought back builds: the builds that
//! served it close with it (and return when it reopens), and a worktree linked to it goes after a
//! short grace.

use std::collections::BTreeSet;

use argus_board::Board;
use argus_board::json::doc;
use argus_board::ops::OpResult;
use argus_board::served::{closure, served_by};
use serde_json::Value;

use crate::verbs::work::run::Run;

/// The builds an op closed with their item and brought back, in the order the op named them.
pub(super) struct Builds {
    pub closed: Vec<String>,
    pub back: Vec<String>,
}

impl Builds {
    /// What an op said it did (`closedWith` and `reopenedWith`).
    pub(super) fn of_result(res: &OpResult) -> Self {
        let names = |key: &str| -> Vec<String> {
            res.extra
                .get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        };
        Self {
            closed: names("closedWith"),
            back: names("reopenedWith"),
        }
    }

    /// What changed between the builds closed by their items before an op and after it.
    pub(super) fn of_sets(was: &BTreeSet<String>, now: &BTreeSet<String>) -> Self {
        Self {
            closed: now.difference(was).cloned().collect(),
            back: was.difference(now).cloned().collect(),
        }
    }
}

fn is_closed(item: Option<&Value>) -> bool {
    item.is_some_and(|i| closure(i).is_some())
}

/// The worktrees linked by hand to an item that just closed.
fn trees(board: &Board, id: &str, before: &Value) -> Option<String> {
    let just_closed = closure(before).is_none() && is_closed(doc(board, "items", id));
    let links = served_by(board, id).worktree_links;
    (just_closed && !links.is_empty()).then(|| {
        format!(
            "worktree {} goes in a short grace (coord worktree link <id> I-n keeps it)",
            links.join(", ")
        )
    })
}

/// Says which builds closed with the item or came back, and which worktree goes with it.
/// `before` is the item as it was.
pub(super) fn told(r: &mut Run<'_>, board: &Board, id: &str, (before, builds): (&Value, &Builds)) {
    let mut parts: Vec<String> = Vec::new();
    if !builds.closed.is_empty() {
        parts.push(format!("build {}", builds.closed.join(", ")));
    }
    parts.extend(trees(board, id, before));
    if !parts.is_empty() {
        r.say(&[&format!("{id}: closed with it: {}.", parts.join("; "))]);
    }
    if !builds.back.is_empty() {
        r.say(&[&format!(
            "{id}: reopened with it: build {}.",
            builds.back.join(", ")
        )]);
    }
}
