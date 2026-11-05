//! The facts the migration reads from legacy results and bundle records.

use serde_json::Value;

use crate::Board;
use crate::json::{display, truthy};
use crate::launchers::build_ready;

/// The summary line a run keeps, as `plan-run-end` writes it; `None` for a summary that is no object.
pub(super) fn counts_line(summary: &Value) -> Option<String> {
    summary.is_object().then(|| {
        let n = |key: &str| display(&summary[key]);
        format!(
            "{} pass, {} fail, {} partial, {} blocked, {} skip, {} not run",
            n("pass"),
            n("fail"),
            n("partial"),
            n("blocked"),
            n("skip"),
            n("notRun"),
        )
    })
}

/// Whether a finished set of results passed: at least one result and the latest of each test passes.
pub(super) fn all_passed(results: &[Value]) -> bool {
    let mut latest: Vec<(&str, &str)> = Vec::new();
    for r in results {
        let (test, status) = (r["test"].as_str(), r["status"].as_str());
        let (Some(test), Some(status)) = (test, status) else {
            continue;
        };
        latest.retain(|(t, _)| *t != test);
        latest.push((test, status));
    }
    !latest.is_empty() && latest.iter().all(|(_, s)| *s == "pass")
}

/// The stage a bundle record has and who it is for: the handed one is ready for the owner, a closed
/// one is passed or reported, an open one is ready while its build is live, else a draft.
pub(super) fn stage_of(
    board: &Board,
    (tag, rec): (&str, &Value),
    results: &[Value],
) -> (&'static str, Option<&'static str>) {
    if truthy(rec, "closedAt") {
        return (if all_passed(results) { "passed" } else { "reported" }, None);
    }
    if board["handedBuild"].as_str() == Some(tag) {
        return ("ready", Some("owner"));
    }
    let ready = build_of(board, rec).is_some_and(|b| build_ready(board, &b));
    (if ready { "ready" } else { "draft" }, None)
}

/// The launcher a bundle record starts, when the board has it.
pub(super) fn build_of(board: &Board, rec: &Value) -> Option<String> {
    let name = rec["launcher"].as_str().filter(|n| !n.is_empty())?;
    board["launchers"][name].is_object().then(|| name.to_owned())
}
