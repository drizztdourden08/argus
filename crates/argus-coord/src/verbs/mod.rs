//! The verb groups served in Rust. A work package adds its folder under `verbs/` (see the layout
//! in the crate documentation) and one row to [`PORTED`]; a group with no row answers exit 120.

mod hook;

use crate::cli::route::Entry;

/// The ported verb groups: the module name of `coord.mjs` and the handler. Every row is one
/// group, so two packages never edit the same row.
pub const PORTED: &[Entry] = &[("hook", hook::run)];
