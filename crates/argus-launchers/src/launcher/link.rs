//! Linking a build to the item it serves (`coord launcher link <name> <I-n>`). The caller has
//! already checked the item against the board; this writes the link, clears a pending relink and
//! leaves the audit line.

use argus_core::build_link::NEEDS_LINK;

use super::audit::audit;
use super::authorize::launcher_to_change;
use super::join::is_master;
use super::record::{LauncherBoard, RuleError, Who};

/// Sets the `for` of a live launcher. A build that was waiting for a link (`relink`) is no longer.
///
/// # Errors
/// No such launcher, a retired one, or a launcher of another thread.
pub fn link_launcher(
    board: &mut LauncherBoard,
    who: &Who,
    (name, item): (&str, &str),
    at: &str,
) -> Result<(), RuleError> {
    let rec = launcher_to_change(board, who, name)?;
    if rec.retired_at.is_some() {
        return Err(RuleError::new(format!(
            "launcher {name} is retired: bring it back with coord launcher set {name} --for {item} instead."
        )));
    }
    if let Some(rec) = board.launchers.get_mut(name) {
        rec.for_item = Some(item.to_owned());
        rec.link.relink = false;
        at.clone_into(&mut rec.updated_at);
        rec.by = Some(who.actor.clone());
    }
    audit(board, who, &format!("linked launcher {name} to {item}"), at);
    Ok(())
}

/// Drops the item a live launcher serves. Only the master build may go without one; a pending
/// relink is cleared with it. Returns the item it served, `None` when it served none.
///
/// # Errors
/// No such launcher, a retired one, a launcher of another thread, or a build that is not the
/// master (every other build needs the item it serves).
pub fn unlink_launcher(
    board: &mut LauncherBoard,
    who: &Who,
    name: &str,
    at: &str,
) -> Result<Option<String>, RuleError> {
    let rec = launcher_to_change(board, who, name)?;
    if rec.retired_at.is_some() {
        return Err(RuleError::new(format!("launcher {name} is retired.")));
    }
    if rec.for_item.is_none() {
        return Ok(None);
    }
    if !is_master(board, name) {
        return Err(RuleError::new(NEEDS_LINK));
    }
    let gone = board.launchers.get_mut(name).and_then(|rec| {
        rec.link.relink = false;
        at.clone_into(&mut rec.updated_at);
        rec.by = Some(who.actor.clone());
        rec.for_item.take()
    });
    if let Some(item) = gone.as_deref() {
        audit(
            board,
            who,
            &format!("unlinked launcher {name} from {item}"),
            at,
        );
    }
    Ok(gone)
}
