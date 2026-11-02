//! Which wakes ask the thread to act, so that one nobody read is a stall. A notice, a fault line, a
//! `stall` and an `ack-nudge` are never in the list.

/// The note one lead sends another with `coord lead`; its ladder runs on the shorter lead steps.
pub const LEAD_NOTES: [&str; 2] = ["lead-info", "lead-message"];

/// The reasons the acknowledgement ladder watches.
pub const ACKABLE: [&str; 9] = [
    "dep-request",
    "item-assigned",
    "owner-answer",
    "owner-note",
    "owner-decision",
    "portfolio-decision",
    "portfolio-note",
    LEAD_NOTES[0],
    LEAD_NOTES[1],
];

/// Does a wake of this reason wait for the thread to read its inbox?
#[must_use]
pub fn is_ackable(reason: &str) -> bool {
    ACKABLE.contains(&reason)
}

/// Is a wake of this reason a `coord lead` note?
#[must_use]
pub fn is_lead_note(reason: &str) -> bool {
    LEAD_NOTES.contains(&reason)
}
