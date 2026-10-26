//! Locks a thread leaves behind: when it is done, archived or purged, the next in the queue gets
//! each one and is told to start.

use std::collections::HashMap;

use argus_board::Board;
use argus_board::json::{display, nullish, section, truthy};
use serde_json::Value;

use super::frame::Frame;

/// Who holds each lock, by resource: compare it before and after a change.
#[must_use]
pub fn lock_holders(board: &Board) -> HashMap<String, Value> {
    section(board, "locks")
        .values()
        .map(|l| (display(&l["resource"]), l["holder"].clone()))
        .collect()
}

/// The line that tells a lead a lock moved to a thread.
pub fn say_pass(frame: &mut Frame<'_>, board: &Board, resource: &str, holder: &str) {
    let session = nullish(&board["threads"][holder], "session").unwrap_or_else(|| "?".to_owned());
    frame.say(&[&format!(
        "lock {resource} passed to {holder}: message them (session \"{session}\") so they start."
    )]);
}

/// Says each lock whose holder is not the one it had before.
pub fn say_passed(frame: &mut Frame<'_>, board: &Board, before: &HashMap<String, Value>) {
    for lock in section(board, "locks").values() {
        let resource = display(&lock["resource"]);
        if truthy(lock, "holder") && before.get(&resource) != Some(&lock["holder"]) {
            say_pass(frame, board, &resource, &display(&lock["holder"]));
        }
    }
}
