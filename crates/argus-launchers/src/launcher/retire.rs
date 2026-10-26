//! Retiring, restoring and keeping launchers.
//!
//! A retired launcher stays on the board (the page lists it under Retired) but has no
//! Play; master is never retired. A launcher made for a bundle goes with the bundle: a close
//! or a removal retires it (`retiredByBundle` remembers a close, so a reopen brings it
//! back); one that another open bundle links as well, and one that merely links the bundle
//! (a shared launcher, master), stays.

use super::audit::audit;
use super::authorize::authorize_launcher;
use super::join::{joined_bundle, linked_bundles, made_for};
use super::play::sync_launcher_play;
use super::record::{LauncherBoard, MASTER, RuleError, Who};
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
    if name == MASTER {
        return Err(RuleError::new(
            "master is never retired: update it with coord launcher set master --cmd \"...\" instead.",
        ));
    }
    let rec = board
        .launchers
        .get(name)
        .ok_or_else(|| RuleError::new(format!("no launcher \"{name}\".")))?;
    authorize_launcher(board, who, Some(rec))?;
    if let Some(rec) = board.launchers.get_mut(name) {
        rec.retired_at = Some(at.to_owned());
        rec.retired_by = Some(who.actor.clone());
        rec.retire_note = note.map(str::to_owned);
        rec.current = false;
        at.clone_into(&mut rec.updated_at);
        rec.by = Some(who.actor.clone());
        rec.retired_by_bundle = None;
    }
    let play = sync_launcher_play(board, who, name, None, at)?;
    let why = note.map(|n| format!(": {n}")).unwrap_or_default();
    audit(board, who, &format!("retired launcher {name}{why}"), at);
    Ok(play)
}

/// A bundle closed (or removed): the launchers made for it go with it. `remember` marks
/// them with the bundle so that a reopen brings them back (a removal marks nothing).
/// Returns the names retired.
pub fn retire_made_for(
    board: &mut LauncherBoard,
    who: &Who,
    tag: &str,
    (why, remember): (&str, bool),
    at: &str,
) -> Result<Vec<String>, RuleError> {
    let mut gone = Vec::new();
    for name in made_for(board, tag) {
        let retired = board
            .launchers
            .get(&name)
            .is_some_and(|l| l.retired_at.is_some());
        let shared = linked_bundles(board, &name, Some(tag))
            .iter()
            .any(|t| board.builds.get(t).is_none_or(|b| b.closed_at.is_none()));
        if retired || shared {
            continue;
        }
        retire_launcher(board, who, &name, Some(why), at)?;
        if let (true, Some(rec)) = (remember, board.launchers.get_mut(&name)) {
            rec.retired_by_bundle = Some(tag.to_owned());
        }
        gone.push(name);
    }
    Ok(gone)
}

/// A bundle reopened: the launchers its close retired come back (one retired some other
/// way stays retired). Returns the names.
pub fn restore_made_for(
    board: &mut LauncherBoard,
    who: &Who,
    tag: &str,
    cwd: Option<&str>,
    at: &str,
) -> Result<Vec<String>, RuleError> {
    let mut back = Vec::new();
    for name in made_for(board, tag) {
        let closed_by_this = board
            .launchers
            .get(&name)
            .is_some_and(|l| l.retired_at.is_some() && l.retired_by_bundle.as_deref() == Some(tag));
        if !closed_by_this {
            continue;
        }
        if let Some(rec) = board.launchers.get_mut(&name) {
            rec.retired_at = None;
            rec.retired_by = None;
            rec.retire_note = None;
            rec.retired_by_bundle = None;
            at.clone_into(&mut rec.updated_at);
            rec.by = Some(who.actor.clone());
        }
        sync_launcher_play(board, who, &name, cwd, at)?;
        audit(
            board,
            who,
            &format!("brought back launcher {name} (bundle {tag} reopened)"),
            at,
        );
        back.push(name);
    }
    Ok(back)
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
    let rec = board
        .launchers
        .get(name)
        .ok_or_else(|| RuleError::new(format!("no launcher \"{name}\".")))?;
    authorize_launcher(board, who, Some(rec))?;
    if rec.retired_at.is_some() {
        return Err(RuleError::new(format!(
            "launcher {name} is retired: there is nothing to keep (coord launcher set {name} brings it back)."
        )));
    }
    if name == MASTER {
        return Err(RuleError::new(
            "master has no usage window: it is never stale.",
        ));
    }
    if let Some(tag) = joined_bundle(board, rec) {
        return Err(RuleError::new(format!(
            "launcher {name} is joined to bundle {tag}: it has no usage window, it goes with the bundle."
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
