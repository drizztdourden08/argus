//! The master role of a board: held by one build at a time and handed over with
//! `coord launcher set <name> --master`, by the coordinator or the owner.

use super::audit::audit;
use super::record::{LauncherBoard, MASTER, RuleError, Who};
use super::write::LauncherPatch;

/// Whether the patch asks for the master role.
pub(super) fn wanted(patch: &LauncherPatch) -> bool {
    patch.master == Some(true)
}

/// Refuses a hand-over by a thread that is not the coordinator or the owner.
pub(super) fn check_hand(who: &Who, patch: &LauncherPatch) -> Result<(), RuleError> {
    if wanted(patch) && !who.is_lead() {
        return Err(RuleError::new(
            "only the coordinator or the owner hands the master role over.",
        ));
    }
    Ok(())
}

/// The builds that hold the role besides `to`: the flagged ones, else the launcher named `master`.
fn holders(board: &LauncherBoard, to: &str) -> Vec<String> {
    let flagged: Vec<String> = board
        .launchers
        .iter()
        .filter(|(name, rec)| rec.link.master && name.as_str() != to)
        .map(|(name, _)| name.clone())
        .collect();
    if !flagged.is_empty() || to == MASTER {
        return flagged;
    }
    board
        .launchers
        .keys()
        .filter(|name| name.as_str() == MASTER)
        .cloned()
        .collect()
}

/// When the patch asked for the role and `to` took it: takes it from the other holders. A build
/// that served nothing is marked `relink`, so its lead links it to an item or retires it.
/// Returns the first holder.
pub(super) fn pass_on(
    board: &mut LauncherBoard,
    patch: &LauncherPatch,
    (to, who, at): (&str, &Who, &str),
) -> Option<String> {
    if !wanted(patch) {
        return None;
    }
    let old = holders(board, to);
    for name in &old {
        if let Some(rec) = board.launchers.get_mut(name) {
            rec.link.master = false;
            rec.link.relink = rec.for_item.is_none();
            at.clone_into(&mut rec.updated_at);
            rec.by = Some(who.actor.clone());
        }
    }
    let first = old.first().cloned();
    if let Some(from) = &first {
        audit(
            board,
            who,
            &format!("handed the master role from {from} to {to}"),
            at,
        );
    }
    first
}
