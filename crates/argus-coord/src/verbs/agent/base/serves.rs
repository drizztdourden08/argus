//! The item a build serves: the `--for` flag, the link rules a write checks, and the test plan a
//! bundle names (the default item of a build made for that bundle).

use argus_board::Board;
use argus_board::json::{section, truthy};
use argus_board::launchers::MASTER;
use argus_board::served::{check_for, is_master};
use argus_launchers::launcher::LauncherPatch;

use super::opt::{Text, given, js_string, not_one_text};
use crate::cli::Options;
use crate::cli::exit::CoordError;

/// The registry type of a test plan.
const PLAN: &str = "test-plan";

/// `--for <I-n>`: the item a build serves. Not given is `None`, blank is `Some(None)` (the master
/// build may go without), else the id.
///
/// # Errors
/// A bare flag or a repeated one.
pub fn for_flag(options: &Options) -> Result<Text, CoordError> {
    if !given(options, "for") {
        return Ok(None);
    }
    if not_one_text(options, "for") {
        return Err(CoordError::refuse(
            "--for takes one item id such as I-12 (\"\" clears it on the master build).",
        ));
    }
    let id = js_string(options, "for").trim().to_owned();
    Ok(Some((!id.is_empty()).then_some(id)))
}

/// Whether `name` is the master build, or is about to be: the launcher named `master` counts
/// while no launcher carries the flag, even before it is made.
fn names_master(board: &Board, name: &str) -> bool {
    let flagged = section(board, "launchers")
        .values()
        .any(|rec| truthy(rec, "master"));
    is_master(board, name) || (name == MASTER && !flagged)
}

/// Checks the item a launcher write names against the board: it must be an item a build may
/// serve. That a non-master build has a link at all is the rule of `write_launcher`, which every
/// write goes through.
///
/// # Errors
/// The sentence of the rule: not an item, a type a build cannot serve, a closed item.
pub fn check_link(board: &Board, name: &str, patch: &LauncherPatch) -> Result<(), CoordError> {
    let Some(Some(id)) = patch.for_item.as_ref() else {
        return Ok(());
    };
    let master = patch.master == Some(true) || names_master(board, name);
    check_for(board, master, Some(id.as_str())).map_err(CoordError::from)
}

/// The ids of the test plans (not in the trash) whose `bundle` field names `tag`.
#[must_use]
pub fn plans_naming(board: &Board, tag: &str) -> Vec<String> {
    section(board, "items")
        .iter()
        .filter(|(_, item)| item["type"] == PLAN && !truthy(item, "deletedAt"))
        .filter(|(_, item)| item["fields"]["bundle"].as_str() == Some(tag))
        .map(|(id, _)| id.clone())
        .collect()
}

/// The one test plan that names `tag`, `None` for none or several.
#[must_use]
pub fn plan_of(board: &Board, tag: &str) -> Option<String> {
    let mut plans = plans_naming(board, tag);
    (plans.len() == 1).then(|| plans.remove(0))
}
