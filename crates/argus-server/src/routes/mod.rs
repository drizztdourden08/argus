//! The routes, one module per area. A later slice adds its module here and drops its entry from
//! `unavailable`.

use crate::Router;

pub mod actions;
pub mod admin;
pub mod db;
pub mod files;
pub mod pages;
mod params;
pub mod unavailable;

pub use params::scan_boards;

/// Registers every route.
pub fn register_all(router: &mut Router) {
    admin::register(router);
    db::register(router);
    actions::register(router);
    pages::register(router);
    files::register(router);
    unavailable::register(router);
}
