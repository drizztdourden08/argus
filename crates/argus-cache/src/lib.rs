//! The build cache of Brock and Tessera, seen from Argus.
//!
//! Brock's `docs/build-cache.md` is the contract: a per-user folder of compiled package versions and Electron
//! binaries that Brock and Tessera only add to. Argus owns usage tracking and pruning. This crate reads the cache,
//! reads the lockfiles of the known projects and worktrees, says who uses each entry, and deletes what no one uses
//! after a grace period. A delete moves the whole `<buildKey>` folder out of sight first, so a reader never meets half
//! an entry, and no path outside the cache root is ever touched.

#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod cache;
mod entry;
mod lockfile;
mod manifest;
mod outcome;
mod places;
mod plan;
mod prune;
mod remove;
mod report;
mod root;
mod scan;
mod segment;
mod tmp;

pub use cache::{BuildCache, PruneError};
pub use entry::{Entry, Kind};
pub use outcome::{Deleted, Outcome};
pub use places::{Place, PlaceKind, User};
pub use report::{Report, Row, Totals};
pub use root::{ENV_DIR, cache_root, default_root};
