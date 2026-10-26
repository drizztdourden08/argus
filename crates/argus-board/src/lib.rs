//! Argus board: the `board.json` model of the coord CLI, ported from JavaScript so both write
//! the same files. It reads, normalizes, locks and saves boards, applies the owner's posts
//! (replies and ops) with the semantics of `coord`, exports a board as page docs and builds the
//! portfolio cards.
#![deny(clippy::cognitive_complexity)]

mod agents;
mod inbox;
mod json;
mod launchers;
mod pause;
mod results;
mod text;

pub mod error;
pub mod limits;
pub mod names;
pub mod ops;
pub mod portfolio;
pub mod proofs;
pub mod store;
pub mod sync;

pub use error::BoardError;

/// A board: always a JSON object. Unknown fields and key order are kept, so the Node CLI and
/// Argus can write the same file.
pub type Board = serde_json::Value;
