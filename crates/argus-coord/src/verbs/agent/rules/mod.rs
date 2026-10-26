//! The launcher and bundle rules of `argus-launchers`, run against the JSON board: the board is
//! read into the typed view the rules take, a rule runs on a copy, and what it changed is written
//! back as the Node CLI writes it (key order, whole Play definitions, audit lines).

mod commit;
mod emit;
mod view;

use argus_board::Board;
use argus_core::clock::now_iso;
use argus_launchers::launcher::{LauncherBoard, RuleError, Who};

use crate::cli::exit::CoordError;
use crate::who::Caller;

pub use view::typed_board;

/// A refusal of a rule as the verb's own.
#[must_use]
pub fn refused(e: RuleError) -> CoordError {
    CoordError::refuse(e.0)
}

/// Runs one rule as `who` on a copy of the board and writes its changes back. The rule gets the
/// caller as the rules know it and the time it stamps.
///
/// # Errors
/// What the rule refuses with, or a record the typed view cannot read.
pub fn run<T>(
    board: &mut Board,
    who: &Caller,
    rule: impl FnOnce(&mut LauncherBoard, &Who, &str) -> Result<T, RuleError>,
) -> Result<T, CoordError> {
    let before = typed_board(board)?;
    let actor = view::rule_who(&before, who);
    let mut after = before.clone();
    let out = rule(&mut after, &actor, &now_iso()).map_err(refused)?;
    commit::commit(board, &before, &after);
    Ok(out)
}
