//! Every reason a wake row can carry, and which of them close the coalescing window at once.
//! Which of them wake a thread at all is the table of [`crate::wake_policy`].

/// The scope of the portfolio page.
pub const PORTFOLIO: &str = "_portfolio";

/// The project whose coordinator is the Overlord.
pub const MAIN_SCOPE: &str = "portfolio";

/// Every reason a wake row can carry.
pub const WAKE_REASONS: [&str; 46] = [
    "test-fail",
    "test-partial",
    "test-blocked",
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
    "fault-open",
    "open-resend",
    "agent-stale",
    "agent-checkin",
    "agent-unassigned",
    "run-failed",
    "agent-orphan",
    "agent-idle",
    "kanban-stale",
    "process-alert",
    "process-reaped",
    "worktree-stale",
    "worktree-review",
    "worktree-unlinked",
    "launcher-stale",
    "launcher-master-missing",
    "remote-sync-due",
    "dep-request",
    "dep-resolved",
    "dep-blocked",
    "dep-cancelled",
    "dep-reopened",
    "ack-nudge",
    "stall",
    "item-unassigned",
    "todo-assigned",
    "item-status",
    "item-ask",
    "item-unblocked",
    "children-done",
    "plan-run-done",
    "todo-quiet",
];

/// What the owner is waiting on, a fault or a stall for the Overlord, a wake sent again because nobody read
/// it, and a finished or blocked dependency: these close the window at once.
pub const IMMEDIATE_REASONS: [&str; 14] = [
    "owner-answer",
    "owner-note",
    "owner-decision",
    "item-assigned",
    "resume",
    "portfolio-decision",
    "portfolio-note",
    "wake-fault",
    "fault-open",
    "run-failed",
    "dep-resolved",
    "dep-blocked",
    "ack-nudge",
    "stall",
];

/// The reasons the wakes route accepts from outside the policy.
pub const EXTERNAL_REASONS: [&str; 4] =
    ["lead-message", "lead-info", "thread-message", "wake-fault"];

/// Does this reason close the window at once?
#[must_use]
pub fn is_immediate(reason: &str) -> bool {
    IMMEDIATE_REASONS.contains(&reason)
}
