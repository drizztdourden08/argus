//! The entry point: one call per mode, the report as lines to print.

use crate::error::Result;
use crate::options::{Mode, Options};
use crate::{plan, remaining, restore, rollback, steps};

fn dry_run(opts: &Options) -> Result<Vec<String>> {
    let home = &opts.home;
    let mut plan = plan::build(home)?;
    plan.remaining = remaining::scan(home, &plan)?;
    plan::save(home, &plan)?;
    let mut lines: Vec<String> = plan
        .boards
        .iter()
        .map(|(key, p)| format!("{key} -> {}  ({})", p.id, p.name))
        .collect();
    let strings: usize = plan.remaining.values().map(|shapes| shapes.len()).sum();
    lines.push(format!(
        "{} boards planned; {strings} kinds of exact-key strings stay, listed per file in {}",
        plan.boards.len(),
        plan::path(home).display()
    ));
    Ok(lines)
}

/// Runs the migration in the mode of `opts`; the lines to print.
///
/// # Errors
/// A precondition or a check stops the run, or a file or the database cannot be read or written.
pub fn run(opts: &Options) -> Result<Vec<String>> {
    match &opts.mode {
        Mode::DryRun => dry_run(opts),
        Mode::Migrate => steps::migrate(opts),
        Mode::Rollback => rollback::run(opts),
        Mode::Restore(from) => restore::run(opts, from),
    }
}
