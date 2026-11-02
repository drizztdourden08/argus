//! The one test of whether a work item is blocked, shared by the stall rules and the stale-item reminders.

use crate::board::{BoardView, ItemView};
use crate::dep_wakes::{RequestedBy, WaitsOn};

/// Is the item blocked: it carries a blocker text, waits on a request to another board, owes the answer to a
/// blocked request, or has an open owner ask?
#[must_use]
pub fn is_blocked(board: &BoardView, item: &ItemView) -> bool {
    item.blocker.some().is_some()
        || waits_across_boards(item)
        || has_open_ask(board, item.rec.id.as_str())
}

fn waits_across_boards(item: &ItemView) -> bool {
    item.waits_on.as_ref().is_some_and(WaitsOn::is_waiting)
        || item
            .requested_by
            .as_ref()
            .is_some_and(RequestedBy::is_blocked)
}

fn has_open_ask(board: &BoardView, item_id: &str) -> bool {
    board.entries.values().any(|e| {
        e.rec.is_live_on(item_id)
            && ((e.kind.is("question") && e.status.is("open"))
                || (e.kind.is("decision") && e.status.is("pending")))
    })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::is_blocked;
    use crate::testkit::{blocking_causes, board};

    fn blocked(item: &Value, entries: &Value) -> bool {
        let mut item = item.clone();
        item["id"] = json!("I-1");
        let board = board(&json!({"items": {"I-1": item}, "entries": entries}));
        is_blocked(&board, &board.items["I-1"])
    }

    fn ask(kind: &str, status: &str) -> Value {
        json!({"E-1": {"item": "I-1", "kind": kind, "status": status}})
    }

    #[test]
    fn a_bare_item_is_not_blocked() {
        assert!(!blocked(&json!({"id": "I-1"}), &json!({})));
    }

    #[test]
    fn each_of_the_four_causes_blocks_an_item() {
        let mut cases = blocking_causes("I-1");
        cases.push((json!({"waitsOn": {"state": "opening"}}), json!({})));
        for (item, entries) in cases {
            assert!(blocked(&item, &entries), "{item} {entries}");
        }
    }

    #[test]
    fn a_settled_state_an_answered_ask_or_an_empty_blocker_does_not() {
        let mut deleted = ask("question", "open");
        deleted["E-1"]["deletedAt"] = json!("2026-10-22T10:00:00.000Z");
        let cases = [
            (json!({"blocker": ""}), json!({})),
            (json!({"waitsOn": {"state": "done"}}), json!({})),
            (json!({"requestedBy": {"state": "open"}}), json!({})),
            (json!({}), ask("question", "answered")),
            (json!({}), deleted),
        ];
        for (item, entries) in cases {
            assert!(!blocked(&item, &entries), "{item} {entries}");
        }
    }
}
