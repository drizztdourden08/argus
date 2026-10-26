//! The routes, one module per area. A service not wired yet has a module with an empty
//! `register`; the run routes still answer 503 from `unavailable` until the runs are wired.

use crate::Router;

pub mod actions;
pub mod activity;
pub mod admin;
mod blocking;
pub mod db;
pub mod files;
pub mod notify;
pub mod pages;
mod params;
pub mod procs;
pub mod unavailable;
pub mod wake;
pub mod worktrees;

pub use params::scan_boards;

/// Registers every route.
pub fn register_all(router: &mut Router) {
    admin::register(router);
    db::register(router);
    actions::register(router);
    activity::register(router);
    procs::register(router);
    wake::register(router);
    notify::register(router);
    worktrees::register(router);
    pages::register(router);
    files::register(router);
    unavailable::register(router);
}
