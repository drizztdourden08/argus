//! The one-way copy job: the chats and folders of one account are copied onto another so the
//! target ends up as the source is. It is the sync a switch runs (current onto next) and a manual
//! action on its own; the two-way merge in [`super::job`] stays as the other manual action.
//!
//! It reuses the sync pipeline: scan, plan, backup, write, then a re-plan that must be empty.
//! Nothing the target alone holds is deleted (see `catalog::plan::onto`).

use serde::{Deserialize, Serialize};

use super::drift::drift_of;
use super::job::{
    Env, Mode, Phase, Planned, Progress, Run, Scan, Summary, kind_of, presence_after, scan,
    write_planned,
};
use super::steps::StepSink;
use crate::catalog::build_plan_onto;
use crate::sidebar::{fresh_folder_id, plan_folders_onto};
use crate::types::Scope;

/// Which way a copy goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    /// The scope that is read; never written.
    pub source: Scope,
    /// The scope that ends up as the source is.
    pub target: Scope,
}

fn plan_onto(env: &Env, scan: Scan, route: &Route) -> Result<Planned, String> {
    let catalog = build_plan_onto(env.catalog, env.transcripts, &route.source, &route.target)?;
    let kinds = catalog.changes.iter().map(|c| kind_of(&scan, c)).collect();
    let presence = presence_after(&scan, &catalog);
    let pair = (route.source.name(), route.target.name());
    let folders = plan_folders_onto(
        &scan.keys,
        &presence,
        (&pair.0, &pair.1),
        &mut fresh_folder_id,
    )?;
    Ok(Planned {
        scan,
        catalog,
        kinds,
        folders,
    })
}

/// Scans and plans a one-way copy.
///
/// # Errors
/// When a record, the sidebar or a plan step cannot be read or made.
pub fn scan_and_plan_onto(env: &Env, route: &Route) -> Result<Planned, String> {
    plan_onto(env, scan(env)?, route)
}

/// Runs the one-way copy. A dry run returns after the plan. An apply refuses while Claude
/// Desktop runs, then backs up, writes the chat files and the sidebar storage and checks that a
/// new plan is empty. On an error the progress callback sees phase `failed`.
///
/// # Errors
/// With a message for the owner: Claude is running, a read or write failed, or the check found
/// changes left (the backup is then named in the message).
pub fn run_onto(
    env: &Env,
    route: &Route,
    mode: Mode,
    sink: &dyn StepSink,
    on_progress: &mut dyn FnMut(&Progress),
) -> Result<Summary, String> {
    let mut run = Run::new(env, sink, on_progress);
    run.start(mode);
    let result = run_inner(&mut run, route, mode);
    run.phase(if result.is_ok() {
        Phase::Done
    } else {
        Phase::Failed
    });
    result
}

fn run_inner(run: &mut Run, route: &Route, mode: Mode) -> Result<Summary, String> {
    let env = run.env;
    let scanned = scan(env)?;
    run.phase(Phase::Plan);
    let planned = plan_onto(env, scanned, route)?;
    let mut summary = Summary {
        mode,
        drift: drift_of(&planned),
        copied: planned.copied(),
        conflicts: planned.catalog.conflicts.len(),
        missing_transcripts: planned.catalog.missing_transcripts.len(),
        verified: false,
        backup_dir: None,
    };
    if mode == Mode::DryRun {
        return Ok(summary);
    }
    if (env.running)() {
        return Err("Claude Desktop is running; close it and run the copy again".to_owned());
    }
    if planned.is_empty() {
        summary.verified = true;
        return Ok(summary);
    }
    let backup_dir = write_planned(run, &planned, &|env| {
        let again = scan_and_plan_onto(env, route)?;
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
