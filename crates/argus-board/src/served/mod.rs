//! Builds served by an item (I-78). A build is a launcher (`board.launchers[name]`) that serves
//! one work item of the board through its `for` field; it lives while the item lives and closes
//! in the same board write when the item reaches the done or dropped lane or goes to the trash,
//! and comes back when the item is reopened or restored. One build per board, the master build,
//! needs no item and never closes with one.
//!
//! * [`served_by`]: what serves an item (builds, closed builds, linked worktrees).
//! * [`check_for`]: the link rules of a new or changed `for`.
//! * [`master`]: the master build, at most one per board and never the one taken away.
//! * [`close_builds`] and [`reopen_builds`]: the closing and the coming back, board fields only
//!   (this crate does not depend on `argus-launchers`; both write the field names of a hand
//!   retire).
//! * [`follow`]: the one call every write of an item makes, from [`crate::store::stamp`].

mod check_for;
mod close;
mod follow;
mod master;
mod play;
mod served_by;
mod state;

// test-tag: critical
#[cfg(test)]
mod tests;

pub use check_for::check_for;
pub use close::{close_builds, closed_by_items, reopen_builds};
pub use follow::follow;
pub use master::{
    check_make_master, check_not_last_master, hand_over_master, is_master, master_name,
};
pub use served_by::{Served, served_by};
pub use state::{closure, for_of};
