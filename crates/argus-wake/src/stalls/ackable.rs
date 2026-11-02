//! Which wakes ask the thread to act, so that one nobody read is a stall. A notice, a fault line, a
//! `stall` and an `ack-nudge` are never in the list.

/// The reasons the acknowledgement ladder watches.
pub const ACKABLE: [&str; 7] = [
    "dep-request",
    "item-assigned",
    "owner-answer",
    "owner-note",
    "owner-decision",
    "portfolio-decision",
    "portfolio-note",
];

/// Does a wake of this reason wait for the thread to read its inbox?
#[must_use]
pub fn is_ackable(reason: &str) -> bool {
    ACKABLE.contains(&reason)
}
