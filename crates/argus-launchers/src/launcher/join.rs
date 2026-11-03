//! The three kinds of launcher.
//!
//! - master: the main checkout's app; mandatory, never retired, no usage window. It is the
//!   launcher with the `master` flag; a board not yet migrated has the launcher named `master`.
//! - served: it carries a `for` link to a work item and goes with the item (I-78).
//! - joined: made for a bundle (`bundle` on the record, set by `coord build --cmd`) or
//!   linked by an open bundle; it goes with the bundle, no usage window.
//! - standalone: everything else on a board not yet migrated (a worktree app, a dev tool);
//!   `useEvery` is mandatory.

use super::record::{Launcher, LauncherBoard, MASTER};

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

/// The tags of the bundles that link the launcher (except `but`).
pub fn linked_bundles(board: &LauncherBoard, name: &str, but: Option<&str>) -> Vec<String> {
    board
        .builds
        .iter()
        .filter(|(tag, bundle)| {
            bundle.launcher.as_deref() == Some(name) && Some(tag.as_str()) != but
        })
        .map(|(tag, _)| tag.clone())
        .collect()
}

/// The bundle a launcher goes with: the one it was made for, else the first open bundle
/// that links it; master is never joined.
pub fn joined_bundle(board: &LauncherBoard, rec: &Launcher) -> Option<String> {
    if is_master(board, &rec.name) {
        return None;
    }
    if rec.bundle.is_some() {
        return rec.bundle.clone();
    }
    linked_bundles(board, &rec.name, None)
        .into_iter()
        .find(|tag| board.builds.get(tag).is_none_or(|b| b.closed_at.is_none()))
}

/// Not the master build, not serving an item and not joined to a bundle: it has a usage window.
pub fn is_standalone(board: &LauncherBoard, rec: &Launcher) -> bool {
    !is_master(board, &rec.name) && rec.for_item.is_none() && joined_bundle(board, rec).is_none()
}

/// The launchers made for a bundle (`coord build --cmd`), master never.
pub fn made_for(board: &LauncherBoard, tag: &str) -> Vec<String> {
    board
        .launchers
        .values()
        .filter(|l| l.bundle.as_deref() == Some(tag) && !is_master(board, &l.name))
        .map(|l| l.name.clone())
        .collect()
}
