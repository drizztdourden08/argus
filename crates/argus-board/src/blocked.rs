//! The one blocked rule: why a work item cannot move, as a list of causes. The wake engine asks
//! whether the list is empty (the stall rules and the stale reminders), the exporter writes the
//! list on each item doc (the card and the pane), so both read the same answer. A link that is
//! settled (done, dropped, passed, answered, deleted, archived) is never a cause.

mod links;
// test-tag: critical
#[cfg(test)]
mod tests;

use serde::Serialize;
use serde_json::Value;

use crate::Board;
use crate::json::{doc, section, truthy};

/// What kind of record holds an item back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CauseKind {
    /// A linked item that is not done or dropped.
    Item,
    /// A linked test that has not passed.
    Test,
    /// A linked to-do that is still pending.
    Todo,
    /// An open question.
    Ask,
    /// A pending decision.
    Decision,
    /// A request to another board that is still being opened or is open.
    DepWaits,
    /// A request of another board, blocked until this item answers.
    DepBlocked,
    /// The old free-text blocker, until the migration turns it into a link.
    Legacy,
}

/// One reason an item is blocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cause {
    /// The kind of record.
    pub kind: CauseKind,
    /// The record id (`I-n`, `T-n`, `H-n`, `Q-n`, `D-n`, a ledger dep id); for the legacy text, the item itself.
    pub id: String,
    /// What to show for it: its title or text.
    pub label: String,
    /// Who holds the ball, when the record names one (a to-do's assignee, the lead of another board).
    pub who: Option<String>,
}

/// Why the item `item_id` is blocked, in the order of the kinds above; empty when nothing blocks it.
#[must_use]
pub fn causes(board: &Board, item_id: &str) -> Vec<Cause> {
    let Some(item) = doc(board, "items", item_id) else {
        return Vec::new();
    };
    let mut found = links::linked(board, item_id, item);
    found.extend(own_asks(board, item_id));
    found.extend(mirrors(item));
    found.extend(legacy(item_id, item));
    found.sort_by_key(|c| c.kind as u8);
    let mut unique: Vec<Cause> = Vec::new();
    for cause in found {
        if !unique.contains(&cause) {
            unique.push(cause);
        }
    }
    unique
}

/// The open questions and pending decisions that belong to the item.
fn own_asks(board: &Board, item_id: &str) -> Vec<Cause> {
    section(board, "entries")
        .iter()
        .filter(|(_, e)| e["item"].as_str() == Some(item_id) && !gone(e))
        .filter_map(|(id, e)| links::open_ask(id, e))
        .collect()
}

/// A record that was deleted or archived.
pub(crate) fn gone(record: &Value) -> bool {
    truthy(record, "deletedAt") || truthy(record, "archivedAt")
}

/// The entries of a mirror field: an object is one entry, an array is its entries, anything else
/// is none.
fn mirror_entries(field: &Value) -> Vec<&Value> {
    match field {
        Value::Array(list) => list.iter().collect(),
        Value::Object(_) => vec![field],
        _ => Vec::new(),
    }
}

/// The requests to and of other boards that hold the item back.
fn mirrors(item: &Value) -> Vec<Cause> {
    let waits = mirror_entries(&item["waitsOn"])
        .into_iter()
        .filter(|m| matches!(m["state"].as_str(), Some("opening" | "open")))
        .map(|m| dep_cause(CauseKind::DepWaits, m));
    let asked = mirror_entries(&item["requestedBy"])
        .into_iter()
        .filter(|m| m["state"].as_str() == Some("blocked"))
        .map(|m| dep_cause(CauseKind::DepBlocked, m));
    waits.chain(asked).collect()
}

fn dep_cause(kind: CauseKind, mirror: &Value) -> Cause {
    let id = mirror["dep"].as_str().unwrap_or_default();
    let label = mirror["title"].as_str().filter(|t| !t.is_empty());
    Cause {
        kind,
        id: id.to_owned(),
        label: label.unwrap_or(id).to_owned(),
        who: mirror["project"]
            .as_str()
            .filter(|p| !p.is_empty())
            .map(|p| format!("lead:{p}")),
    }
}

/// The free-text blocker of a board that has not run the migration.
fn legacy(item_id: &str, item: &Value) -> Option<Cause> {
    let text = item["blocker"].as_str().filter(|t| !t.is_empty())?;
    Some(Cause {
        kind: CauseKind::Legacy,
        id: item_id.to_owned(),
        label: text.to_owned(),
        who: None,
    })
}
