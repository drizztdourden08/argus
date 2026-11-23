//! The run of a review: scan, plan the owner's choices and edits, then the sync job's backup,
//! writes and verify. The verify reads back every file the run wrote (and the sidebar storage)
//! and compares it with the plan, since a review may leave differences on purpose.

use serde_json::json;

use super::decide::{Request, plan_review};
use crate::catalog::merge::{json_bytes, sha256_hex};
use crate::catalog::plan::{Change, ChangeKind};
use crate::catalog::read::tombstone_name;
use crate::sidebar::{SidebarKeys, fresh_folder_id};
use crate::sync::drift::drift_of;
use crate::sync::job::{Env, Mode, Phase, Planned, Progress, Run, Summary, scan, write_planned};
use crate::sync::steps::StepSink;

/// Runs a review: refuses while Claude Desktop runs, backs up, writes, verifies. On an error the
/// progress callback sees phase `failed`.
///
/// # Errors
/// With a message for the owner: a row, choice or edit is wrong, Claude is running, a read or
/// write failed, or what was written differs from the plan (the backup is named then).
pub fn run_review(
    env: &Env,
    request: &Request,
    sink: &dyn StepSink,
    on_progress: &mut dyn FnMut(&Progress),
) -> Result<Summary, String> {
    let mut run = Run::new(env, sink, on_progress);
    run.start(Mode::Apply);
    let result = run_inner(&mut run, request);
    run.phase(if result.is_ok() {
        Phase::Done
    } else {
        Phase::Failed
    });
    result
}

fn run_inner(run: &mut Run, request: &Request) -> Result<Summary, String> {
    let env = run.env;
    let scanned = scan(env)?;
    run.phase(Phase::Plan);
    let planned = plan_review(env, scanned, request, &mut fresh_folder_id)?;
    let mut summary = Summary {
        mode: Mode::Apply,
        drift: drift_of(&planned),
        copied: planned.copied(),
        conflicts: planned.catalog.conflicts.len(),
        missing_transcripts: planned.catalog.missing_transcripts.len(),
        verified: false,
        backup_dir: None,
    };
    if (env.running)() {
        return Err("Claude Desktop is running; close it and apply again".to_owned());
    }
    if !planned.is_empty() {
        let dir = write_planned(run, &planned, &|env| check_written(env, &planned))?;
        summary.backup_dir = Some(dir);
    }
    summary.verified = true;
    Ok(summary)
}

/// Every file the plan wrote holds what the plan says.
fn check_written(env: &Env, planned: &Planned) -> Result<(), String> {
    let mut wrong = Vec::new();
    for change in &planned.catalog.changes {
        if !written(env, change)? {
            wrong.push(format!("{}/{}", change.scope.name(), change.file));
        }
    }
    if !planned.folders.steps.is_empty() {
        let now = SidebarKeys::from_value(&env.sidebar.read()?)?;
        let want = &planned.folders.keys;
        let same = now.code_sections.get("value") == want.code_sections.get("value")
            && now.group_scopes.get("value") == want.group_scopes.get("value")
            && now.dframe_store == want.dframe_store;
        if !same {
            wrong.push("the sidebar storage".to_owned());
        }
    }
    match wrong.as_slice() {
        [] => Ok(()),
        _ => Err(format!(
            "{} written files differ from the plan: {}",
            wrong.len(),
            wrong.join(", ")
        )),
    }
}

fn written(env: &Env, change: &Change) -> Result<bool, String> {
    let catalog = env.catalog;
    let sha = |file: &str| catalog.file_sha256(&change.scope, file);
    Ok(match change.kind {
        ChangeKind::CopySession | ChangeKind::UpdateSession => {
            sha(&change.file)? == Some(sha256_hex(&json_bytes(&change.data)))
        }
        ChangeKind::ArchiveIndex => {
            Some(&json!(catalog.archived(&change.scope)?)) == change.data.get("archived")
        }
        ChangeKind::DeleteSession => {
            sha(&change.file)?.is_none() && sha(&tombstone_name(&change.session_id))?.is_some()
        }
    })
}
