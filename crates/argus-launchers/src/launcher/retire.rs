//! Retiring, restoring and keeping launchers.
//!
//! A retired launcher stays on the board (the page lists it under Retired) but has no
//! Play; master is never retired. A launcher made for a bundle goes with the bundle: a close
//! or a removal retires it (`retiredByBundle` remembers a close, so a reopen brings it
//! back); one that another open bundle links as well, and one that merely links the bundle
//! (a shared launcher, master), stays.

use super::audit::audit;
use super::authorize::launcher_to_change;
use super::join::{is_master, joined_bundle};
use super::play::sync_launcher_play;
use super::record::{LauncherBoard, RuleError, Who};
use crate::time::{DAY_MS, format_iso, parse_iso};

/// Retires a launcher: no Play, off the list, no longer current. Returns the Play note;
/// refuses master.
pub fn retire_launcher(
    board: &mut LauncherBoard,
    who: &Who,
    name: &str,
    note: Option<&str>,
    at: &str,
) -> Result<String, RuleError> {
    if is_master(board, name) {
        return Err(RuleError::new(format!(
            "{name} is never retired: update it with coord launcher set {name} --cmd \"...\" instead."
        )));
    }
    launcher_to_change(board, who, name)?;
    if let Some(rec) = board.launchers.get_mut(name) {
        rec.retired_at = Some(at.to_owned());
        rec.retired_by = Some(who.actor.clone());
        rec.retire_note = note.map(str::to_owned);
        rec.current = false;
        at.clone_into(&mut rec.updated_at);
        rec.by = Some(who.actor.clone());
        rec.retired_by_bundle = None;
        rec.retired_by_item = None;
    }
    let play = sync_launcher_play(board, who, name, None, at)?;
    let why = note.map(|n| format!(": {n}")).unwrap_or_default();
    audit(board, who, &format!("retired launcher {name}{why}"), at);
    Ok(play)
}

/// `coord launcher keep`: counts as a use without running it. `days` holds it fresh until
/// then; without, one more full window from now. Records who and why. Only a standalone,
/// live launcher has a window to keep.
pub fn keep_launcher(
    board: &mut LauncherBoard,
    who: &Who,
    name: &str,
    (days, note): (Option<i64>, Option<&str>),
    at: &str,
) -> Result<(), RuleError> {
    let rec = launcher_to_change(board, who, name)?;
    if rec.retired_at.is_some() {
        return Err(RuleError::new(format!(
            "launcher {name} is retired: there is nothing to keep (coord launcher set {name} brings it back)."
        )));
    }
    if is_master(board, name) {
        return Err(RuleError::new(format!(
            "{name} has no usage window: it is never stale."
        )));
    }
    if let Some(tag) = joined_bundle(board, rec) {
        return Err(RuleError::new(format!(
            "launcher {name} carries bundle tag {tag}: it has no usage window."
        )));
    }
    let until = days.map(|d| format_iso(parse_iso(at).unwrap_or(0) + d * DAY_MS));
    if let Some(rec) = board.launchers.get_mut(name) {
        rec.kept_at = Some(at.to_owned());
        rec.kept_by = Some(who.actor.clone());
        rec.kept_note = note.map(str::to_owned);
        rec.kept_until = until;
    }
    let span = days.map(|d| format!(" for {d}d")).unwrap_or_default();
    let why = note.map(|n| format!(": {n}")).unwrap_or_default();
    audit(board, who, &format!("kept launcher {name}{span}{why}"), at);
    Ok(())
}
