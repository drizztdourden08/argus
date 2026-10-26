//! WP5: `coord agent`, `coord build` and `coord launcher`: the sub-agents a thread runs, the
//! bundles of tests the owner is handed, and the launchers the owner can start.
//!
//! ```text
//! base/       how a verb reaches the board, options, ownership rules, sort order
//! rules/      the launcher and bundle rules of `argus-launchers` run on the JSON board
//! agents/     agent: start, note, set, done, fail, stop, archive, unarchive, rm, restore,
//!             list, show, reconcile
//! builds/     build: add, set, rm, close, reopen, ready, draft, hand, unhand, list
//! launchers/  launcher: set, keep, retire, list, show
//! ```

mod agents;
pub mod base;
mod builds;
mod launchers;
mod rules;

use crate::cli::Call;
use crate::cli::exit::Outcome;

/// The `agent` verb.
///
/// # Errors
/// The refusal of the sub-verb.
pub fn agent(call: &mut Call<'_>) -> Outcome {
    agents::run(call)
}

/// The `build` verb.
///
/// # Errors
/// The refusal of the sub-verb.
pub fn build(call: &mut Call<'_>) -> Outcome {
    builds::run(call)
}

/// The `launcher` verb.
///
/// # Errors
/// The refusal of the sub-verb.
pub fn launcher(call: &mut Call<'_>) -> Outcome {
    launchers::run(call)
}
