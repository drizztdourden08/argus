//! The forward run, in the order of the plan: marker, server check, locks, backup, table, boards,
//! portfolio, database, home files, verification, then `active`. Every step checks the disk before
//! it acts, so the same call after an interruption finishes the job; a finished run only
//! verifies again.

use std::path::{Path, PathBuf};

use argus_core::ids::PORTFOLIO;

use crate::error::{Result, stop};
use crate::journal::Journal;
use crate::locks::Held;
use crate::map::{self, IdMap};
use crate::options::{Fault, Options};
use crate::plan::{self, Plan, board_file};
use crate::verify::{self, Check};
use crate::{backup, fault, files, locks, marker, precheck, relocate, rewrite, sql, table};

fn any_renamed(home: &Path, journal: &Journal) -> bool {
    journal
        .boards
        .iter()
        .any(|e| e.phase == "renamed" || home.join(&e.id).is_dir())
}

fn ensure_backup(opts: &Options, journal: &mut Journal) -> Result<PathBuf> {
    let kept = journal.backup.as_ref().map(PathBuf::from);
    if let Some(path) = kept.as_ref().filter(|p| p.is_dir()) {
        backup::check_folder(path)?;
        return Ok(path.clone());
    }
    if let Some(gone) = kept {
        if any_renamed(&opts.home, journal) {
            return stop(format!(
                "the backup {} is gone and boards are already renamed: a new backup would hold a half-migrated home; restore the folder or roll back",
                gone.display()
            ));
        }
    }
    let path = backup::make(opts)?;
    journal.backup = Some(path.display().to_string());
    journal.save(&opts.home)?;
    Ok(path)
}

fn move_boards(opts: &Options, plan: &Plan, ids: &IdMap, journal: &mut Journal) -> Result<()> {
    for (n, (key, planned)) in plan.boards.iter().enumerate() {
        relocate::move_one(&opts.home, key, &planned.id, ids)?;
        journal.phase(&opts.home, key, "renamed")?;
        fault::at(opts, Fault::AfterBoard(n + 1))?;
    }
    Ok(())
}

fn rewrite_shared(opts: &Options, ids: &IdMap) -> Result<()> {
    rewrite::portfolio::rewrite_file(&board_file(&opts.home, PORTFOLIO), ids)?;
    sql::rewrite(&opts.home.join(backup::DB_REL), ids)?;
    fault::at(opts, Fault::AfterDb)?;
    files::rewrite(&opts.home, ids)
}

/// What the refusals before the first change leave behind.
struct Ready {
    plan: Plan,
    journal: Journal,
    backup: PathBuf,
    _locks: Held,
}

/// Everything that can refuse before the first change: server, plan, locks, backup.
fn prepare(opts: &Options, saved: Option<Journal>) -> Result<Ready> {
    let home = &opts.home;
    precheck::server_down(opts)?;
    let plan = plan::build(home)?;
    plan::save(home, &plan)?;
    let held = locks::take(home, &map::pairs(&plan))?;
    precheck::server_down(opts)?;
    let mut journal = saved.unwrap_or_else(|| Journal::new(&plan));
    journal.ensure(&plan);
    let backup = ensure_backup(opts, &mut journal)?;
    Ok(Ready {
        plan,
        journal,
        backup,
        _locks: held,
    })
}

fn cut_over(opts: &Options, ready: Ready) -> Result<Vec<String>> {
    let home = &opts.home;
    let Ready {
        plan,
        mut journal,
        backup,
        _locks,
    } = ready;
    let ids = map::forward(&plan);
    table::seed(home, &plan)?;
    move_boards(opts, &plan, &ids, &mut journal)?;
    rewrite_shared(opts, &ids)?;
    let check = Check {
        home,
        entries: &journal.boards,
        map: &ids,
        backup: Some(&backup),
    };
    verify::run(&check)?;
    table::activate(home)?;
    journal.complete = true;
    journal.save(home)?;
    marker::clear(home);
    Ok(report(&plan, &backup))
}

fn report(plan: &Plan, backup: &Path) -> Vec<String> {
    let mut lines = vec![
        format!("migrated {} boards to ids", plan.boards.len()),
        format!("backup: {}", backup.display()),
    ];
    if let Ok(manifest) = backup::read_manifest(backup) {
        lines.push(format!(
            "the backup left out {} links or junctions (listed in its manifest.json, recreate them by hand after a restore) and {} lock files",
            manifest.links.len(),
            manifest.locks_skipped
        ));
    }
    lines
}

fn finished(home: &Path, journal: &Journal) -> Result<Vec<String>> {
    let ids: IdMap = journal
        .boards
        .iter()
        .map(|e| (e.key.clone(), e.id.clone()))
        .collect();
    verify::run(&Check {
        home,
        entries: &journal.boards,
        map: &ids,
        backup: None,
    })?;
    Ok(vec!["already migrated; verified again, nothing changed".to_owned()])
}

/// Migrates, finishes an interrupted migration, or verifies a finished one.
pub fn migrate(opts: &Options) -> Result<Vec<String>> {
    let home = &opts.home;
    let saved = Journal::load(home)?;
    if let Some(done) = saved.as_ref().filter(|j| j.complete) {
        return finished(home, done);
    }
    let keep = saved.is_some() || marker::exists(home);
    marker::take(opts)?;
    match prepare(opts, saved) {
        Ok(ready) => cut_over(opts, ready),
        Err(e) => Err(marker::release(home, keep, e)),
    }
}
