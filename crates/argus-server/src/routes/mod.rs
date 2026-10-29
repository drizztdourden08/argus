//! The routes, one module per area. A service not wired yet has a module with an empty
//! `register`.

use crate::Router;

pub mod actions;
pub mod activity;
pub mod admin;
pub mod agents;
mod blocking;
pub mod cache;
pub mod db;
pub mod feed;
pub mod files;
pub mod mcp;
pub mod notify;
pub mod pages;
mod params;
pub mod procs;
pub mod records;
pub mod runs;
pub mod wake;
pub mod worktrees;

pub use params::scan_boards;

/// Registers every route.
pub fn register_all(router: &mut Router) {
    admin::register(router);
    db::register(router);
    actions::register(router);
    activity::register(router);
    agents::register(router);
    procs::register(router);
    runs::register(router);
    wake::register(router);
    notify::register(router);
    worktrees::register(router);
    cache::register(router);
    records::register(router);
    feed::register(router);
    pages::register(router);
    files::register(router);
    mcp::register(router);
}
