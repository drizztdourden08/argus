//! The pull request ops of the local remote (`docs/local-remote.md`): a PR is a board record
//! `PR-n` in the `prs` collection with a head branch, a base, the head commit, its status and the
//! `replay` block that says where it stands on the way to GitHub. `pr-open` makes one,
//! `pr-update` moves its head commit or text, `pr-merge` and `pr-close` end it, and `pr-checks`
//! (Argus itself) keeps the list of the check runs that belong to it. The merge itself is a
//! local git merge done by Brock's tooling; this only records it.

mod change;
mod fields;
mod open;

use serde_json::Value;

use super::result::OpResult;
use super::who::Who;
use crate::Board;
use crate::error::{BoardError, refuse};

/// Whether an op kind is one of the pull request ops.
#[must_use]
pub fn is_pr_kind(kind: &str) -> bool {
    matches!(
        kind,
        "pr-open" | "pr-update" | "pr-merge" | "pr-close" | "pr-checks"
    )
}

/// Apply one pull request op.
pub(super) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    match op["kind"].as_str() {
        Some("pr-open") => open::handle(board, op, who),
        Some("pr-update") => change::update(board, op, who),
        Some("pr-merge") => change::merge(board, op, who),
        Some("pr-close") => change::close(board, op, who),
        Some("pr-checks") => change::checks(board, op, who),
        _ => Err(refuse("not a pull request op.")),
    }
}
