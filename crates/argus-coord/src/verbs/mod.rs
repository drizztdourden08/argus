//! The verb groups served in Rust. A work package adds its folder under `verbs/` (see the layout
//! in the crate documentation) and one row to [`PORTED`]; a group with no row answers exit 120.

pub mod work;

use crate::cli::route::Entry;

/// The ported verb groups: the module name of `coord.mjs` and the handler. Every row is one
/// group, so two packages never edit the same row.
pub const PORTED: &[Entry] = &[
    ("item", work::item),
    ("test", work::test),
    ("entry", work::entry),
    ("action", work::action),
    ("lock", work::lock),
    ("log", work::log),
    ("inbox", work::inbox),
];
