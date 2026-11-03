//! Linking a build to the item it serves (`coord launcher link <name> <I-n>`). The caller has
//! already checked the item against the board; this writes the link, clears a pending relink and
//! leaves the audit line.

use super::audit::audit;
use super::authorize::authorize_launcher;
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
    let rec = board
        .launchers
        .get(name)
        .ok_or_else(|| RuleError::new(format!("no launcher \"{name}\".")))?;
    authorize_launcher(board, who, Some(rec))?;
    if rec.retired_at.is_some() {
        return Err(RuleError::new(format!(
            "launcher {name} is retired: bring it back with coord launcher set {name} --for {item} instead."
        )));
    }
    if let Some(rec) = board.launchers.get_mut(name) {
        rec.for_item = Some(item.to_owned());
        rec.relink = false;
        at.clone_into(&mut rec.updated_at);
        rec.by = Some(who.actor.clone());
    }
    audit(board, who, &format!("linked launcher {name} to {item}"), at);
    Ok(())
}
