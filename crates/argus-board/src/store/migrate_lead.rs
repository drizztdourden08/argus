//! The lead rename: a board written before it names the lead thread `coordinator`. The thread key
//! (and `coordinator-hosted`), the keys of its log and inbox, and every person or thread field
//! that holds exactly that id become `lead`. Prose is never touched. `board.migrations.lead`
//! marks the board as done, so a second run changes nothing.

use argus_core::lead::{LEAD, LEGACY, canonical};
use serde_json::{Map, Value};

use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{map_mut, section, slot, truthy};

const HOSTED: &str = "coordinator-hosted";

/// The fields that hold a person or a thread id.
const PERSON_FIELDS: [&str; 11] = [
    "assignee",
    "thread",
    "createdBy",
    "updatedBy",
    "by",
    "actor",
    "origin",
    "reporter",
    "to",
    "from",
    "role",
];

/// The collections keyed by thread id.
const KEYED: [&str; 3] = ["threads", "log", "inbox"];

/// Whether the board still names the lead `coordinator` and has not been migrated.
pub(crate) fn pending(board: &Board) -> bool {
    !truthy(&board["migrations"], "lead") && has_legacy_thread(board)
}

fn has_legacy_thread(board: &Board) -> bool {
    let threads = section(board, "threads");
    threads.contains_key(LEGACY) || threads.contains_key(HOSTED)
}

/// Refuse a board whose `lead` thread is not the old coordinator.
///
/// # Errors
/// The board names a thread `lead` next to a `coordinator` one.
pub(crate) fn check(board: &Board) -> Result<(), BoardError> {
    if pending(board) && section(board, "threads").contains_key(LEAD) {
        return Err(refuse(
            "the board has a thread named \"lead\" next to \"coordinator\"; rename or remove it first, then reload.",
        ));
    }
    Ok(())
}

/// Migrate the board once. A board that cannot be migrated is left untouched (`check` says why).
pub(crate) fn migrate(board: &mut Board) {
    if truthy(&board["migrations"], "lead") || check(board).is_err() {
        return;
    }
    for coll in KEYED {
        rename_keys(slot(board, coll));
    }
    rename_counters(slot(board, "counters"));
    rewrite(board);
    map_mut(board, "migrations").insert("lead".to_owned(), Value::Bool(true));
}

fn rename_keys(coll: &mut Value) {
    let Some(map) = coll.as_object_mut() else {
        return;
    };
    for old in [LEGACY, HOSTED] {
        if let Some(doc) = map.remove(old) {
            map.insert(canonical(old).into_owned(), doc);
        }
    }
}

fn rename_counters(counters: &mut Value) {
    let Some(map) = counters.as_object_mut() else {
        return;
    };
    for old in [LEGACY, HOSTED] {
        let key = format!("log:{old}");
        if let Some(n) = map.remove(&key) {
            map.insert(format!("log:{}", canonical(old)), n);
        }
    }
}

fn rewrite(v: &mut Value) {
    match v {
        Value::Object(map) => rewrite_object(map),
        Value::Array(items) => items.iter_mut().for_each(rewrite),
        _ => {}
    }
}

fn rewrite_object(map: &mut Map<String, Value>) {
    for (key, value) in map.iter_mut() {
        let person = PERSON_FIELDS.contains(&key.as_str()) || key == "subject" || key == "id";
        match value {
            Value::String(s) if person => {
                if let Some(id) = swapped(s) {
                    *s = id;
                }
            }
            other => rewrite(other),
        }
    }
}

fn swapped(s: &str) -> Option<String> {
    let id = canonical(s);
    (id != s).then(|| id.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn old_board() -> Board {
        json!({
            "project": "p",
            "counters": {"log:coordinator": 2, "I": 1},
            "threads": {
                "coordinator": {"id": "coordinator", "role": "coordinator", "note": "the coordinator leads"},
                "coordinator-hosted": {"id": "coordinator-hosted", "role": "coordinator"},
                "ui": {"id": "ui", "role": "thread"},
            },
            "items": {"I-1": {"assignee": "coordinator", "thread": "ui", "createdBy": "coordinator",
                "updatedBy": "ui", "origin": "coordinator", "title": "coordinator", "description": "ask the coordinator"}},
            "inbox": {"coordinator": [{"from": "ui", "text": "hi coordinator"}], "ui": [{"from": "coordinator", "to": "ui"}]},
            "log": {"coordinator": {"subject": "coordinator", "entries": [{"thread": "coordinator", "text": "coordinator"}]}},
            "entries": {"E-1": {"by": "coordinator", "actor": "coordinator", "reporter": "coordinator", "text": "coordinator"}},
        })
    }

    #[test]
    fn a_coordinator_board_comes_out_with_lead_everywhere_and_prose_untouched() {
        let mut board = old_board();
        assert!(pending(&board));
        migrate(&mut board);
        assert_eq!(board["threads"]["lead"]["id"], "lead");
        assert_eq!(board["threads"]["lead"]["role"], "lead");
        assert_eq!(board["threads"]["lead"]["note"], "the coordinator leads");
        assert_eq!(board["threads"]["lead-hosted"]["id"], "lead-hosted");
        assert_eq!(board["counters"]["log:lead"], 2);
        assert_eq!(board["items"]["I-1"]["assignee"], "lead");
        assert_eq!(board["items"]["I-1"]["createdBy"], "lead");
        assert_eq!(board["items"]["I-1"]["origin"], "lead");
        assert_eq!(board["items"]["I-1"]["title"], "coordinator");
        assert_eq!(board["items"]["I-1"]["description"], "ask the coordinator");
        assert_eq!(board["inbox"]["lead"][0]["text"], "hi coordinator");
        assert_eq!(board["inbox"]["ui"][0]["from"], "lead");
        assert_eq!(board["log"]["lead"]["subject"], "lead");
        assert_eq!(board["log"]["lead"]["entries"][0]["thread"], "lead");
        assert_eq!(board["log"]["lead"]["entries"][0]["text"], "coordinator");
        assert_eq!(board["entries"]["E-1"]["by"], "lead");
        assert_eq!(board["entries"]["E-1"]["text"], "coordinator");
        assert_eq!(board["migrations"]["lead"], true);
        assert!(board["threads"].get("coordinator").is_none());
        assert!(!pending(&board));
    }

    #[test]
    fn a_second_run_changes_nothing() {
        let mut board = old_board();
        migrate(&mut board);
        let once = board.clone();
        migrate(&mut board);
        assert_eq!(board, once);
    }

    #[test]
    fn a_lead_thread_next_to_a_coordinator_is_refused() {
        let mut board = json!({"threads": {"coordinator": {"id": "coordinator"}, "lead": {"id": "lead", "role": "thread"}}});
        let before = board.clone();
        let err = check(&board).expect_err("refused");
        assert!(err.to_string().contains("\"lead\""), "{err}");
        migrate(&mut board);
        assert_eq!(board, before);
    }

    #[test]
    fn a_board_without_a_coordinator_is_only_marked() {
        let mut board = json!({"threads": {"lead": {"id": "lead", "role": "lead"}}});
        assert!(!pending(&board));
        migrate(&mut board);
        assert_eq!(board["migrations"]["lead"], true);
        assert_eq!(board["threads"]["lead"]["id"], "lead");
    }
}
