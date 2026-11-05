//! The three kinds of launcher.
//!
//! - master: the main checkout's app; mandatory, never retired, no usage window. It is the
//!   launcher with the `master` flag; a board not yet migrated has the launcher named `master`.
//! - served: it carries a `for` link to a work item and goes with the item (I-78).
//! - legacy joined: a record that still carries a bundle tag; no usage window.
//! - standalone: everything else on a board not yet migrated (a worktree app, a dev tool);
//!   `useEvery` is mandatory.
//!
//! A build made for a test run is a served launcher: `for` names the run, and
//! `served::close_builds` retires it when the run ends.

use super::record::{Launcher, LauncherBoard, MASTER, RuleError};

pub fn is_live(rec: Option<&Launcher>) -> bool {
    rec.is_some_and(|r| r.retired_at.is_none())
}

/// Whether `name` is the master build: the flagged launcher, or the one named `master` while no
/// launcher carries the flag.
pub fn is_master(board: &LauncherBoard, name: &str) -> bool {
    if board.launchers.values().any(|l| l.link.master) {
        board.launchers.get(name).is_some_and(|l| l.link.master)
    } else {
        name == MASTER
    }
}

/// Every project must have a live master launcher.
pub fn has_master(board: &LauncherBoard) -> bool {
    if board.launchers.values().any(|l| l.link.master) {
        board
            .launchers
            .values()
            .any(|l| l.link.master && l.retired_at.is_none())
    } else {
        is_live(board.launchers.get(MASTER))
    }
}

/// The legacy bundle tag a launcher was made for (`bundle` on the record). Nothing sets it
/// any more; a launcher that still carries one keeps no usage window. Master is never joined.
pub fn joined_bundle(board: &LauncherBoard, rec: &Launcher) -> Option<String> {
    if is_master(board, &rec.name) {
        return None;
    }
    rec.bundle.clone()
}

/// Not the master build, not serving an item and not carrying a legacy bundle tag: it has a
/// usage window.
pub fn is_standalone(board: &LauncherBoard, rec: &Launcher) -> bool {
    !is_master(board, &rec.name) && rec.for_item.is_none() && joined_bundle(board, rec).is_none()
}

/// A test run is handed to the owner only while the project has a master launcher.
pub fn require_master(board: &LauncherBoard) -> Result<(), RuleError> {
    if has_master(board) {
        return Ok(());
    }
    Err(RuleError::new(format!(
        "this project has no master launcher, so nothing can be handed to the owner yet: every project must have one (the main checkout's app). Set master first: coord launcher set {MASTER} --cmd \"<command that starts it>\"."
    )))
}
