//! The records of the local remote (`docs/local-remote.md`): what Brock writes under
//! `~/.coord/releases` and `~/.coord/ci`, what the Overlord's `localpkg` keeps, and the update
//! feed that serves the release store. Everything here only reads; the files are the truth and
//! each store keeps an index that a folder watch refreshes.
#![deny(clippy::cognitive_complexity)]

pub mod checks;
mod fields;
pub mod folder_watch;
pub mod registry;
pub mod releases;
mod replay;
pub mod safe_path;
mod stamp;

pub use replay::{Replay, replay_json, replay_of};
