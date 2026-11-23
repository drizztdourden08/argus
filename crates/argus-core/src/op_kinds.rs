//! The kinds of op an entry may carry: the owner's edits, moves, archives, deletes and purges,
//! the worktree and build ops, and the test plan run ops. Each is an entry with its own target
//! and fields, applied to the board through the same ops the CLI uses.

/// Every op kind an entry may carry.
pub const OP_KINDS: [&str; 21] = [
    "create",
    "edit",
    "move",
    "ac",
    "archive",
    "unarchive",
    "delete",
    "restore",
    "bulk",
    "purge",
    "build",
    "avatar",
    "icon",
    "kanban-config",
    "worktree-link",
    "worktree-keep",
    "launcher-link",
    "launcher-retire",
    "plan-run-start",
    "plan-run-record",
    "plan-run-end",
];

/// Whether an entry kind is an op kind.
#[must_use]
pub fn is_op_kind(kind: &str) -> bool {
    OP_KINDS.contains(&kind)
}
