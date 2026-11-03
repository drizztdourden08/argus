//! The routes, one module per area. A service not wired yet has a module with an empty
//! `register`.

use crate::Router;

pub mod actions;
pub mod activity;
pub mod admin;
pub mod agents;
mod blocking;
pub mod cache;
mod check_guard;
pub mod check_runs;
pub mod db;
pub mod deps;
pub mod feed;
pub mod files;
pub mod inbox;
pub mod item_images;
pub mod mcp;
pub mod mux;
pub mod notify;
pub mod pages;
mod params;
pub mod procs;
pub mod prs;
pub mod records;
pub mod runs;
pub mod stalls;
pub mod trail;
pub mod trails;
pub mod wake;
pub mod worktrees;
mod write;

pub use params::scan_boards;

/// Registers every route.
pub fn register_all(router: &mut Router) {
    admin::register(router);
    db::register(router);
    deps::register(router);
    actions::register(router);
    item_images::register(router);
    activity::register(router);
    agents::register(router);
    procs::register(router);
    runs::register(router);
    wake::register(router);
    inbox::register(router);
    stalls::register(router);
    trail::register(router);
    trails::register(router);
    notify::register(router);
    worktrees::register(router);
    cache::register(router);
    records::register(router);
    prs::register(router);
    check_runs::register(router);
    feed::register(router);
    pages::register(router);
    files::register(router);
    mcp::register(router);
    mux::register(router);
}
