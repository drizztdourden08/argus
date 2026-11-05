//! Writing what the rules changed back onto the JSON board: only the records that differ, only
//! the keys that differ (a key keeps its place, a new key goes last), the Play definitions as
//! whole objects and the audit lines the rules wrote.

use argus_board::Board;
use argus_board::json::{map_mut, set, unset};
use argus_board::store::{LogLine, append_log};
use argus_launchers::launcher::LauncherBoard;
use serde_json::{Map, Value};

use super::emit::{def_map, launcher_map};

/// Sets the keys of `after` that differ from `before` on the record, and drops the keys `after`
/// no longer has.
fn diff(rec: &mut Value, before: &Map<String, Value>, after: &Map<String, Value>) {
    for (key, value) in after {
        if before.get(key) != Some(value) {
            set(rec, key, value.clone());
        }
    }
    for key in before.keys().filter(|k| !after.contains_key(*k)) {
        unset(rec, key);
    }
}

/// Puts one record of a collection: a changed one by its keys, a new one whole.
fn put(
    board: &mut Board,
    coll: &str,
    key: &str,
    after: Map<String, Value>,
    before: Option<Map<String, Value>>,
) {
    let records = map_mut(board, coll);
    match (records.get_mut(key), before) {
        (Some(rec), Some(before)) => diff(rec, &before, &after),
        _ => {
            records.insert(key.to_owned(), Value::Object(after));
        }
    }
}

fn launchers(board: &mut Board, before: &LauncherBoard, after: &LauncherBoard) {
    for (name, rec) in &after.launchers {
        let was = before.launchers.get(name);
        if was != Some(rec) {
            put(
                board,
                "launchers",
                name,
                launcher_map(rec),
                was.map(launcher_map),
            );
        }
    }
}

fn definitions(board: &mut Board, before: &LauncherBoard, after: &LauncherBoard) {
    for (name, def) in &after.run_defs {
        if before.run_defs.get(name) != Some(def) {
            map_mut(board, "runDefs").insert(name.clone(), Value::Object(def_map(def)));
        }
    }
    for name in before
        .run_defs
        .keys()
        .filter(|n| !after.run_defs.contains_key(*n))
    {
        map_mut(board, "runDefs").shift_remove(name);
    }
}

/// Writes the difference between the rules' board before and after onto the JSON board.
pub fn commit(board: &mut Board, before: &LauncherBoard, after: &LauncherBoard) {
    launchers(board, before, after);
    definitions(board, before, after);
    for line in &after.audit {
        let entry = LogLine {
            thread: &line.thread,
            item: None,
            text: &line.text,
        };
        append_log(board, "audit", &entry);
    }
}
