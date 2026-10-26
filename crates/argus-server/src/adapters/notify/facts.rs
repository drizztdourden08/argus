//! What the automatic pings read of an exported board.

use argus_board::Board;
use argus_core::js::truthy;
use argus_notify::{BoardFacts, Bundle, EntryFact, TestFact};
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

fn test_of(test: &Value) -> TestFact {
    TestFact {
        build: text_of(&test["build"]),
        removed: removed(test),
    }
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

fn bundle_of(board: &Board, tag: Option<&str>) -> Option<Bundle> {
    let bundle = &board["builds"][tag?];
    bundle.is_object().then(|| Bundle {
        title: text_of(&bundle["title"]),
        thread: text_of(&bundle["thread"]),
    })
}

/// The facts of the board of `key`; `portfolio` is the board that lists the projects and says
/// which ones are retired.
#[must_use]
pub fn facts_of(key: &str, board: &Board, portfolio: Option<&Board>) -> BoardFacts {
    let tag = text_of(&board["handedBuild"]).filter(|tag| !tag.is_empty());
    BoardFacts {
        retired: portfolio.is_some_and(|p| truthy(&p["projects"][key]["retired"])),
        handed_at: text_of(&board["handedAt"]),
        bundle: bundle_of(board, tag.as_deref()),
        handed_build: tag,
        tests: values(&board["tests"]).map(test_of).collect(),
        entries: values(&board["entries"]).map(entry_of).collect(),
    }
}
