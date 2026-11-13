//! Step 8: the checks. The full pass (the end of a real run) compares every board, the portfolio
//! and every table with the backup; the light pass (a finished run asked again) checks only the
//! shape, because writes made since the cutover are legitimate.

mod boards;
mod db;
mod proofs;

use std::path::Path;

use crate::error::{Result, stop};
use crate::journal::Entry;
use crate::map::IdMap;

/// What a verification looks at.
pub struct Check<'a> {
    /// The coord home.
    pub home: &'a Path,
    /// The boards of the run.
    pub entries: &'a [Entry],
    /// Old key to id.
    pub map: &'a IdMap,
    /// The verified backup, for the full pass.
    pub backup: Option<&'a Path>,
}

/// Runs the checks; the failures, all of them, come back in one message.
pub fn run(check: &Check) -> Result<()> {
    let mut problems: Vec<String> = Vec::new();
    boards::check(check, &mut problems)?;
    db::check(check, &mut problems)?;
    if problems.is_empty() {
        return Ok(());
    }
    stop(format!("verification failed:\n{}", problems.join("\n")))
}
