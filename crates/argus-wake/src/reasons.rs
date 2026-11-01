//! Every reason a wake row can carry, and which of them close the coalescing window at once.
//! Which of them wake a thread at all is the table of [`crate::wake_policy`].

/// The scope of the portfolio page.
pub const PORTFOLIO: &str = "_portfolio";

/// The project whose coordinator is the Overlord.
pub const MAIN_SCOPE: &str = "portfolio";

/// Every reason a wake row can carry.
pub const WAKE_REASONS: [&str; 28] = [
    "test-fail",
    "test-partial",
    "test-blocked",
    "bundle-complete",
    "owner-answer",
    "owner-note",
    "owner-decision",
    "item-assigned",
    "resume",
    "portfolio-decision",
    "portfolio-note",
    "lead-message",
    "lead-info",
    "thread-message",
    "wake-fault",
    "agent-stale",
    "agent-checkin",
    "run-failed",
    "agent-orphan",
    "agent-idle",
    "kanban-stale",
    "process-alert",
    "process-reaped",
    "worktree-stale",
    "worktree-review",
    "launcher-stale",
    "launcher-master-missing",
    "remote-sync-due",
];

/// What the owner is waiting on, and a fault for the Overlord: these close the window at once.
pub const IMMEDIATE_REASONS: [&str; 9] = [
    "owner-answer",
    "owner-note",
    "owner-decision",
    "item-assigned",
    "resume",
    "portfolio-decision",
    "portfolio-note",
    "wake-fault",
    "run-failed",
];

/// The reasons the wakes route accepts from outside the policy.
pub const EXTERNAL_REASONS: [&str; 4] =
    ["lead-message", "lead-info", "thread-message", "wake-fault"];

/// Does this reason close the window at once?
#[must_use]
pub fn is_immediate(reason: &str) -> bool {
    IMMEDIATE_REASONS.contains(&reason)
}
