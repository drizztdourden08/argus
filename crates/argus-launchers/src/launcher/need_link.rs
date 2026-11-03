//! The link rule of a build write: every build except the master serves an item. A new build
//! without `for` is refused, and so is a write that clears the link of one. Which item may be
//! served (open, of the right type, on this board) is checked against the board by the caller.

use argus_core::build_link::NEEDS_LINK;

use super::join::is_master;
use super::master_role;
use super::record::{Launcher, LauncherBoard, RuleError};
use super::write::LauncherPatch;

/// Whether the write leaves the build without an item to serve.
fn leaves_unlinked(have: Option<&Launcher>, patch: &LauncherPatch) -> bool {
    match &patch.for_item {
        Some(wanted) => wanted.is_none(),
        None => have.is_none(),
    }
}

/// Refuses a new non-master build without `for`, and a write that clears the link of one.
pub(super) fn check(
    board: &LauncherBoard,
    name: &str,
    have: Option<&Launcher>,
    patch: &LauncherPatch,
) -> Result<(), RuleError> {
    let master = is_master(board, name) || master_role::wanted(patch);
    if !master && leaves_unlinked(have, patch) {
        return Err(RuleError::new(NEEDS_LINK));
    }
    Ok(())
}
