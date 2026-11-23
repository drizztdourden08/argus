//! Argus board: the `board.json` model of the coord CLI, ported from JavaScript so both write
//! the same files. It reads, normalizes, locks and saves boards, applies the owner's posts
//! (replies and ops) with the semantics of `coord`, exports a board as page docs and builds the
//! portfolio cards and keeps the ledger of dependencies between projects.
#![deny(clippy::cognitive_complexity)]

pub mod agent_links;
pub mod agents;
pub mod blocked;
pub mod content;
pub mod demo;
pub mod deps;
pub mod error;
pub mod handed;
pub mod holds;
pub mod ids;
pub mod inbox;
pub mod item_events;
pub mod json;
pub mod kanban_config;
pub mod launchers;
pub mod limits;
pub mod migrate_runs;
pub mod names;
pub mod ops;
pub mod pause;
pub mod portfolio;
pub mod proofs;
pub mod registry;
pub mod results;
pub mod roles;
pub mod served;
pub mod store;
pub mod sync;
pub mod text;
pub mod title_limit;
pub mod trail;

pub use error::BoardError;
pub use item_events::item_events;

/// A board: always a JSON object. Unknown fields and key order are kept, so the Node CLI and
/// Argus can write the same file.
pub type Board = serde_json::Value;
