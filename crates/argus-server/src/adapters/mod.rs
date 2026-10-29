//! Thin adapters between the traits a service crate defines and the server's own services:
//! its store over the doc store and the tables, its host over the boards, the wake engine and
//! the activity log. One module per service crate, written when that crate is wired; the
//! activity sinks of every crate are in [`activity`].

pub mod activity;
pub mod agents;
pub mod check_links;
pub mod mcp;
pub mod notify;
pub mod procs;
pub mod records;
pub mod runs;
pub mod stale;
pub mod wake;
pub mod worktrees;
