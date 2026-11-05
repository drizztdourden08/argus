//! What the automatic pings read of an exported board.

use argus_board::Board;
use argus_board::handed::is_handed_run;
use argus_core::js::truthy;
use argus_notify::{BoardFacts, EntryFact, HandedRun};
use serde_json::Value;

fn text_of(v: &Value) -> Option<String> {
    v.as_str().map(str::to_owned)
}

fn removed(v: &Value) -> bool {
    truthy(&v["deletedAt"]) || truthy(&v["archivedAt"])
}

fn values(v: &Value) -> impl Iterator<Item = &Value> {
    v.as_object().into_iter().flat_map(|o| o.values())
}

fn entry_of(entry: &Value) -> EntryFact {
    EntryFact {
        id: text_of(&entry["id"]).unwrap_or_default(),
        kind: text_of(&entry["kind"]).unwrap_or_default(),
        status: text_of(&entry["status"]).unwrap_or_default(),
        author: text_of(&entry["author"]).unwrap_or_default(),
        needs_decision: entry["needs"] == "decision",
        thread: text_of(&entry["thread"]),
        text: text_of(&entry["text"]).unwrap_or_default(),
        created_at: text_of(&entry["createdAt"]),
        removed: removed(entry),
    }
}

fn handed_of(board: &Board, run: &Value) -> HandedRun {
    let plan = &run["fields"]["plan"];
    HandedRun {
        id: text_of(&run["id"]).unwrap_or_default(),
        title: text_of(&run["title"]),
        thread: text_of(&run["thread"]),
        at: text_of(&run["updatedAt"]),
        tests: values(&board["tests"])
            .filter(|test| test["item"] == *plan && !removed(test))
            .count(),
    }
}

/// The run handed to the owner: the one changed last.
fn handed_run(board: &Board) -> Option<HandedRun> {
    values(&board["items"])
        .filter(|run| is_handed_run(board, run))
        .max_by_key(|run| text_of(&run["updatedAt"]))
        .map(|run| handed_of(board, run))
}

/// The facts of the board of `key`; `portfolio` is the board that lists the projects and says
/// which ones are retired.
#[must_use]
pub fn facts_of(key: &str, board: &Board, portfolio: Option<&Board>) -> BoardFacts {
    BoardFacts {
        retired: portfolio.is_some_and(|p| truthy(&p["projects"][key]["retired"])),
        handed: handed_run(board),
        entries: values(&board["entries"]).map(entry_of).collect(),
    }
}
