//! The one test of whether a work item is blocked, shared by the stall rules and the stale-item reminders.

use argus_board::blocked::causes;

use crate::board::{BoardView, ItemView};

/// Is the item blocked: does the shared rule of `argus-board` find a cause (a blocker link, a request to or of another
/// board, an open ask, or the old blocker text)? Any one cause is enough, so an item is released only when all of
/// them are settled.
#[must_use]
pub fn is_blocked(board: &BoardView, item: &ItemView) -> bool {
    !causes(board.json(), item.rec.id.as_str()).is_empty()
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
    fn an_item_waiting_on_several_requests_is_blocked_until_all_are_done() {
        let waits = |states: [&str; 2]| json!({"waitsOn": [{"dep": "D-1", "state": states[0]}, {"dep": "D-2", "state": states[1]}]});
        assert!(blocked(&waits(["done", "open"]), &json!({})));
        assert!(!blocked(&waits(["done", "done"]), &json!({})));
    }

    #[test]
    fn an_item_requested_by_several_boards_is_blocked_while_one_request_is_blocked() {
        let asked = |states: [&str; 2]| json!({"requestedBy": [{"dep": "D-1", "state": states[0]}, {"dep": "D-2", "state": states[1]}]});
        assert!(blocked(&asked(["open", "blocked"]), &json!({})));
        assert!(!blocked(&asked(["open", "done"]), &json!({})));
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
