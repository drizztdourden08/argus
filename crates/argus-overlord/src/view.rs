//! The rows of the owner's Queue view, built from what the server reads: held messages, pending
//! wakes, and the alerts (open notices of the Overlord's board and stall lines). All pure.

use serde_json::{Value, json};

use crate::session::inbox::Held;

/// A held message is cut to this many characters in the view.
pub const HELD_CLIP: usize = 500;

fn clip(text: &str) -> String {
    text.chars().take(HELD_CLIP).collect()
}

/// `{text, at}` for each held message, the text cut to [`HELD_CLIP`] characters.
#[must_use]
pub fn held_rows(held: &[Held]) -> Vec<Value> {
    held.iter()
        .map(|h| json!({ "text": clip(&h.text), "at": h.at }))
        .collect()
}

/// `{reason, text, at}` of one pending wake.
#[must_use]
pub fn wake_row(reason: Option<&str>, text: Option<&str>, at: &str) -> Value {
    json!({ "reason": reason, "text": text.map(clip), "at": at })
}

/// `{kind: "stall", project, text, at}` of one open stall.
#[must_use]
pub fn stall_row(project: &str, text: &str, since: &str) -> Value {
    json!({ "kind": "stall", "project": project, "text": text, "at": since })
}

fn is_open_notice(entry: &Value) -> bool {
    let kind = entry["kind"].as_str();
    matches!(kind, Some("notice" | "message")) && entry["status"] == "open"
}

/// `{kind: "notice", id, text, level, at}` for each open notice or message entry of a board, oldest
/// first. A board with no entries has none.
#[must_use]
pub fn notice_rows(board: &Value) -> Vec<Value> {
    let Some(entries) = board["entries"].as_object() else {
        return Vec::new();
    };
    let mut open: Vec<&Value> = entries.values().filter(|e| is_open_notice(e)).collect();
    open.sort_by_key(|e| e["createdAt"].as_str().unwrap_or_default());
    open.into_iter()
        .map(|e| {
            json!({
                "kind": "notice",
                "id": e["id"],
                "text": e["text"].as_str().map(clip),
                "level": e["level"],
                "at": e["createdAt"],
            })
        })
        .collect()
}

// test-tag: critical
#[cfg(test)]
mod specs {
    use super::*;

    #[test]
    fn a_held_text_is_cut_and_keeps_its_time() {
        let held = [Held {
            text: "x".repeat(HELD_CLIP + 40),
            at: "2026-11-22T10:00:00Z".to_owned(),
        }];
        let rows = held_rows(&held);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["text"].as_str().map(str::len), Some(HELD_CLIP));
        assert_eq!(rows[0]["at"], "2026-11-22T10:00:00Z");
    }

    #[test]
    fn only_open_notices_and_messages_are_alerts_oldest_first() {
        let board = json!({ "entries": {
            "N-2": { "id": "N-2", "kind": "notice", "status": "open", "text": "b", "level": "warn", "createdAt": "2026-11-22T11:00:00Z" },
            "N-1": { "id": "N-1", "kind": "message", "status": "open", "text": "a", "createdAt": "2026-11-22T10:00:00Z" },
            "N-3": { "id": "N-3", "kind": "notice", "status": "dismissed", "text": "c" },
            "Q-1": { "id": "Q-1", "kind": "question", "status": "open", "text": "d" }
        } });
        let rows = notice_rows(&board);
        let ids: Vec<&str> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
        assert_eq!(ids, ["N-1", "N-2"]);
        assert_eq!(rows[1]["level"], "warn");
    }

    #[test]
    fn a_board_without_entries_has_no_alerts() {
        assert_eq!(notice_rows(&json!({})).len(), 0);
    }

    #[test]
    fn a_wake_row_clips_its_text_and_keeps_a_missing_reason_null() {
        let row = wake_row(None, Some("hello"), "t");
        assert_eq!(row["reason"], Value::Null);
        assert_eq!(row["text"], "hello");
    }
}
