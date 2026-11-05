//! What `build migrate-links` reads from the board: the audit line it writes and the live tests
//! that still carry a legacy build tag.

use argus_board::Board;
use argus_board::json::{section, truthy};
use argus_board::store::{LogLine, append_log};
use argus_board::text::clip_text;
use serde_json::Value;

use crate::who::Caller;

const AUDIT_LINE: usize = 120;

/// One audit line of the board, as the actor.
pub fn audit(board: &mut Board, who: &Caller, text: &str) {
    let line = clip_text(text, AUDIT_LINE);
    let entry = LogLine {
        thread: &who.actor,
        item: None,
        text: &line,
    };
    append_log(board, "audit", &entry);
}

/// A test that counts: not in the trash, not archived.
fn live(test: &Value) -> bool {
    !truthy(test, "deletedAt") && !truthy(test, "archivedAt")
}

/// The live tests that carry a legacy build tag.
#[must_use]
pub fn carrying<'a>(board: &'a Board, tag: &str) -> Vec<&'a Value> {
    section(board, "tests")
        .values()
        .filter(|t| t["build"].as_str() == Some(tag) && live(t))
        .collect()
}
