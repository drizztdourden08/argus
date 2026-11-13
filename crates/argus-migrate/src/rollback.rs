//! `--rollback`: reverses steps 7 to 3 from the journal with the same whitelist (id to key), so
//! what was written after the cutover is kept. The database, the portfolio and the home files go
//! back first, then every folder, then the table and the journal go.

use std::fs;
use std::io;
use std::path::Path;

use argus_core::boards;
use argus_core::ids::PORTFOLIO;

use crate::error::{Result, stop};
use crate::journal::{self, Journal};
use crate::map;
use crate::options::Options;
use crate::plan::board_file;
use crate::{backup, files, locks, marker, precheck, relocate, rewrite, sql};

fn strangers(home: &Path, journal: &Journal) -> Result<()> {
    let table = boards::load(home)?;
    let extra: Vec<&str> = table
        .boards
        .keys()
        .filter(|id| !journal.boards.iter().any(|e| e.id == **id))
        .map(String::as_str)
        .collect();
    if extra.is_empty() {
        return Ok(());
    }
    stop(format!(
        "boards made after the cutover have no old key ({}): a rollback would lose them, nothing was changed",
        extra.join(", ")
    ))
}

fn remove(file: &Path) -> Result<()> {
    match fs::remove_file(file) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

/// Reverses the migration of the home.
pub fn run(opts: &Options) -> Result<Vec<String>> {
    let home = &opts.home;
    let Some(journal) = Journal::load(home)? else {
        return stop("there is no board-ids.journal.json: nothing to roll back");
    };
    precheck::server_down(opts)?;
    strangers(home, &journal)?;
    marker::take(opts)?;
    let pairs: Vec<(String, String)> = journal
        .boards
        .iter()
        .map(|e| (e.key.clone(), e.id.clone()))
        .collect();
    let _locks = locks::take(home, &pairs).inspect_err(|_| marker::clear(home))?;
    let back = map::reverse(&journal.boards);
    files::rewrite(home, &back)?;
    sql::rewrite(&home.join(backup::DB_REL), &back)?;
    rewrite::portfolio::rewrite_file(&board_file(home, PORTFOLIO), &back)?;
    for e in &journal.boards {
        relocate::move_one(home, &e.id, &e.key, &back)?;
    }
    remove(&home.join("boards.json"))?;
    remove(&journal::path(home))?;
    marker::clear(home);
    Ok(vec![format!(
        "rolled back {} boards to their keys; writes since the cutover are kept",
        journal.boards.len()
    )])
}
