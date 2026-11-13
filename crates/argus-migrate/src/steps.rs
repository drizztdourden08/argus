//! The forward run, in the order of the plan: precondition, marker and locks, backup, table,
//! boards, portfolio, database, home files, verification, then `active`. Every step checks the
//! disk before it acts, so the same call after an interruption finishes the job; a finished run
//! only verifies again.

use std::path::{Path, PathBuf};

use argus_core::ids::PORTFOLIO;

use crate::error::Result;
use crate::journal::Journal;
use crate::map::{self, IdMap};
use crate::options::{Fault, Options};
use crate::plan::{self, Plan, board_file};
use crate::verify::{self, Check};
use crate::{backup, fault, files, locks, marker, precheck, relocate, rewrite, sql, table};

fn ensure_backup(opts: &Options, journal: &mut Journal) -> Result<PathBuf> {
    let kept = journal
        .backup
        .as_ref()
        .map(PathBuf::from)
        .filter(|p| p.is_dir());
    if let Some(path) = kept {
        backup::check_folder(&path)?;
        return Ok(path);
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

fn cut_over(opts: &Options, plan: &Plan, journal: &mut Journal) -> Result<Vec<String>> {
    let home = &opts.home;
    let backup = match ensure_backup(opts, journal) {
        Ok(path) => path,
        Err(e) => {
            marker::clear(home);
            return Err(e);
        }
    };
    let ids = map::forward(plan);
    table::seed(home, plan)?;
    move_boards(opts, plan, &ids, journal)?;
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
    Ok(vec![
        format!("migrated {} boards to ids", plan.boards.len()),
        format!("backup: {}", backup.display()),
    ])
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
    precheck::server_down(opts)?;
    let plan = plan::build(home)?;
    plan::save(home, &plan)?;
    marker::take(opts)?;
    let _locks = locks::take(home, &map::pairs(&plan)).inspect_err(|_| marker::clear(home))?;
    let mut journal = saved.unwrap_or_else(|| Journal::new(&plan));
    journal.ensure(&plan);
    cut_over(opts, &plan, &mut journal)
}
