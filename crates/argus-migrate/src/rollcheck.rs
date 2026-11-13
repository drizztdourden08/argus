//! The rollback precheck: every board and every database key the rollback would write is looked
//! at before the first write. Any problem stops the run with the whole list.

use std::path::Path;

use crate::error::{Result, stop};
use crate::journal::Journal;
use crate::map::IdMap;
use crate::{backup, relocate, sql};

fn board_problem(home: &Path, key: &str, id: &str) -> Option<String> {
    let (at_id, at_key) = (home.join(id).is_dir(), home.join(key).is_dir());
    let stray = relocate::is_stray(home, key) || relocate::is_stray(home, id);
    match (at_id, at_key) {
        (true, true) if !stray => Some(format!("both {key} and {id} exist as board folders")),
        (false, false) => Some(format!("neither {key} nor {id} exists as a board folder")),
        _ => None,
    }
}

/// Stops with the list of boards and database rows a rollback would trip over.
pub fn run(home: &Path, journal: &Journal, back: &IdMap) -> Result<()> {
    let mut problems: Vec<String> = journal
        .boards
        .iter()
        .filter_map(|e| board_problem(home, &e.key, &e.id))
        .collect();
    problems.extend(sql::collisions(&home.join(backup::DB_REL), back)?);
    if problems.is_empty() {
        return Ok(());
    }
    stop(format!(
        "the rollback would collide, nothing was changed:\n{}",
        problems.join("\n")
    ))
}
