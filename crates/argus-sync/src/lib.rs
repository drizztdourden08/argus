//! The sync service (DESIGN 4): every board under `COORD_HOME` mirrored live into the doc store,
//! owner posts applied under the board lock and exported before the answer, proofs kept with the
//! board, and the portfolio cards of the Argus overview.
#![deny(clippy::cognitive_complexity)]

mod exporter;
mod failure;
mod hooks;
mod portfolio;
mod posts;
mod proofs;
mod prs;
mod queue;
mod read;
mod runs;
mod service;
mod state;
mod validate;
mod watcher;

pub use failure::board_error;
pub use hooks::{AppliedPost, Hooks};
pub use posts::{PostResult, Source, new_post_id};
pub use proofs::{Proof, ProofSource, sniff_image};
pub use prs::PrApplied;
pub use queue::ScopeQueue;
pub use runs::RunsView;
pub use service::{ProjectRow, Resynced, SyncOptions, SyncService, WatcherHealth};
pub use validate::RESULT_NOTE_REQUIRED;
