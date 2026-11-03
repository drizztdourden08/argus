//! The names of a stall in the book, the kv and the page documents.

use crate::stall_rules::Found;

/// `<project>:<kind>:<subject>:<sinceMs>`. A new occurrence has a new `since`, so an old ack never covers it.
pub(super) fn stall_key(found: &Found) -> String {
    format!(
        "{}:{}:{}:{}",
        found.project,
        found.kind.as_str(),
        found.subject,
        found.since
    )
}

/// The kv key of the ack of a stall.
pub(super) fn ack_key(stall: &str) -> String {
    format!("stall-ack:{stall}")
}

/// The kv key of the last time a thread read its own inbox.
pub(super) fn read_key(scope: &str, thread: &str) -> String {
    format!("inbox-read:{scope}:{thread}")
}
