//! The frame every sync run shares: report the start and the end phase, open the summary of a
//! plan, and the apply half of a copy or merge (refuse while Claude runs, back up, write, check
//! that a new plan is empty).

use super::drift::drift_of;
use super::job::{Env, Mode, Phase, Planned, Progress, Run, Summary, write_planned};
use super::steps::StepSink;

/// Runs `body` as one run: reports the start, then phase `done` or, on an error, `failed`.
pub(super) fn drive(
    env: &Env,
    mode: Mode,
    sink: &dyn StepSink,
    on_progress: &mut dyn FnMut(&Progress),
    body: impl FnOnce(&mut Run) -> Result<Summary, String>,
) -> Result<Summary, String> {
    let mut run = Run::new(env, sink, on_progress);
    run.start(mode);
    let result = body(&mut run);
    run.phase(if result.is_ok() {
        Phase::Done
    } else {
        Phase::Failed
    });
    result
}

/// The summary of a plan before anything is written.
pub(super) fn opening_summary(mode: Mode, planned: &Planned) -> Summary {
    Summary {
        mode,
        drift: drift_of(planned),
        copied: planned.copied(),
        conflicts: planned.catalog.conflicts.len(),
        missing_transcripts: planned.catalog.missing_transcripts.len(),
        verified: false,
        backup_dir: None,
    }
}

/// The apply of a plan: a dry run returns the summary, an apply refuses while Claude Desktop
/// runs (`retry` names what to run again), writes and checks that `replan` finds nothing left.
pub(super) fn apply_planned(
    run: &mut Run,
    planned: &Planned,
    mut summary: Summary,
    retry: &str,
    replan: &dyn Fn(&Env) -> Result<Planned, String>,
) -> Result<Summary, String> {
    let env = run.env;
    if summary.mode == Mode::DryRun {
        return Ok(summary);
    }
    if (env.running)() {
        return Err(format!(
            "Claude Desktop is running; close it and run the {retry} again"
        ));
    }
    if planned.is_empty() {
        summary.verified = true;
        return Ok(summary);
    }
    let backup_dir = write_planned(run, planned, &|env| {
        let again = replan(env)?;
        let left = again.catalog.changes.len() + again.folders.steps.len();
        if left > 0 {
            return Err(format!("{left} changes are still planned"));
        }
        Ok(())
    })?;
    summary.backup_dir = Some(backup_dir);
    summary.verified = true;
    Ok(summary)
}
