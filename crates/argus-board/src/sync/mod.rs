//! The bridge between the board file and the dashboard. The page only ever adds documents (the
//! owner's entries); it never edits one a thread wrote. Everything the owner does on the page is
//! an entry: a reply, a pause, or an op (create, edit, move, ac, archive, unarchive, delete,
//! restore, bulk, purge, build) applied to the board through the same ops the CLI uses.
//!
//! * [`export_docs`]: the board as page documents.
//! * [`import_entries`]: the owner's new entries applied to the board, under the board lock.

mod agent_docs;
mod export;
mod import;
mod repeat;
mod replies;

use argus_core::Coll;

pub use argus_core::op_kinds::OP_KINDS;
pub use export::export_docs;
pub use import::{ImportResult, OpCounts, import_entries, import_entries_at};

/// The collections the page mirrors from a board.
pub const PAGE_COLLECTIONS: [Coll; 11] = [
    Coll::Threads,
    Coll::Items,
    Coll::Tests,
    Coll::Entries,
    Coll::Locks,
    Coll::Actions,
    Coll::Agents,
    Coll::Log,
    Coll::Builds,
    Coll::Launchers,
    Coll::Prs,
];

/// What each reply kind of owner entry does to the board.
pub const REPLY_KINDS: [&str; 8] = [
    "pause", "resume", "followup", "decision", "ack", "done", "result", "rerun",
];

/// Whether an entry kind is a reply kind.
#[must_use]
pub fn is_reply_kind(kind: &str) -> bool {
    REPLY_KINDS.contains(&kind)
}

/// Whether an entry kind is an op kind.
#[must_use]
pub fn is_op_kind(kind: &str) -> bool {
    argus_core::op_kinds::is_op_kind(kind)
}
