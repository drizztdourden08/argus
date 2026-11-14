//! The verb groups served in Rust. A work package adds its folder under `verbs/` (see the layout
//! in the crate documentation) and one row to [`PORTED`]; a group with no row answers exit 120.

pub mod agent;
mod demo;
mod hook;
pub mod remote;
pub mod shared;
pub mod thread;
pub mod work;

use crate::cli::route::Entry;

/// The ported verb groups: the module name of `coord.mjs` and the handler. Every row is one
/// group, so two packages never edit the same row.
pub const PORTED: &[Entry] = &[
    ("agent", agent::agent),
    ("build", agent::build),
    ("launcher", agent::launcher),
    ("hook", hook::run),
    ("demo", demo::seed),
    ("thread", thread::run_thread),
    ("dispatch", thread::run_dispatch),
    ("board", thread::run_board),
    ("purge", thread::run_purge),
    ("check", remote::ci::handle),
    ("pr", remote::pr::handle),
    ("tick", thread::run_tick),
    ("sync", thread::run_sync),
    ("item", work::item),
    ("test", work::test),
    ("types", work::types),
    ("entry", work::entry),
    ("action", work::action),
    ("lock", work::lock),
    ("log", work::log),
    ("inbox", work::inbox),
    ("run", remote::run::handle),
    ("remote", remote::mode::handle),
    ("ping", remote::ping::handle),
    ("activity", remote::activity::handle),
    ("proc", remote::proc::handle),
    ("wake", remote::wake::handle),
    ("wakefault", remote::wakefault::handle),
    ("stall", remote::stall::handle),
    ("worktree", remote::worktree::handle),
    ("lead", remote::lead::handle),
    ("portfolio", remote::portfolio::handle),
    ("dashboard", remote::dashboard::handle),
    ("handover", remote::handover::handle),
];
