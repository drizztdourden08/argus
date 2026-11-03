//! Argus board: the `board.json` model of the coord CLI, ported from JavaScript so both write
//! the same files. It reads, normalizes, locks and saves boards, applies the owner's posts
//! (replies and ops) with the semantics of `coord`, exports a board as page docs and builds the
//! portfolio cards and keeps the ledger of dependencies between projects.
#![deny(clippy::cognitive_complexity)]

pub mod agents;
pub mod blocked;
pub mod content;
pub mod deps;
pub mod error;
pub mod inbox;
pub mod json;
pub mod launchers;
pub mod limits;
pub mod names;
pub mod ops;
pub mod pause;
pub mod portfolio;
pub mod proofs;
pub mod registry;
pub mod results;
pub mod store;
pub mod sync;
pub mod text;

pub use error::BoardError;

/// A board: always a JSON object. Unknown fields and key order are kept, so the Node CLI and
/// Argus can write the same file.
pub type Board = serde_json::Value;
