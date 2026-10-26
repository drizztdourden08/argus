//! The bundle writes that touch launchers: close and reopen (which retire and restore the
//! launchers made for the bundle), ready and draft, and the rules for handing a build to
//! the owner (a master launcher is mandatory).

use std::fmt::Write as _;

use super::audit::audit;
use super::bundle::{BundleStatus, bundle_status};
use super::join::has_master;
use super::record::{Bundle, LauncherBoard, MASTER, RuleError, Who};
use super::retire::{restore_made_for, retire_made_for};

/// The record of a tag, made when missing.
pub fn ensure_bundle<'a>(
    board: &'a mut LauncherBoard,
    tag: &str,
    actor: &str,
    at: &str,
) -> &'a mut Bundle {
    board
        .builds
        .entry(tag.to_owned())
        .or_insert_with(|| Bundle {
            tag: tag.to_owned(),
            created_at: Some(at.to_owned()),
            created_by: Some(actor.to_owned()),
            updated_at: Some(at.to_owned()),
            ..Bundle::default()
        })
}

/// What a close needs to know besides the board.
#[derive(Clone, Copy, Debug, Default)]
pub struct CloseRequest<'a> {
    pub note: Option<&'a str>,
    pub force: bool,
    /// Ids of the manual tests of the tag nobody has run yet.
    pub pending_tests: &'a [String],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseOutcome {
    /// The bundle was closed already; nothing changed.
    pub already: bool,
    pub unhanded: bool,
    /// The launchers made for the bundle that went with it.
    pub retired: Vec<String>,
}

/// Closes a bundle: its tests leave the owner's Tests tab and the launchers made for it
/// are retired with it. Refused while manual tests of the tag are pending unless forced.
pub fn close_bundle(
    board: &mut LauncherBoard,
    who: &Who,
    tag: &str,
    request: &CloseRequest<'_>,
    at: &str,
) -> Result<CloseOutcome, RuleError> {
    if board.builds.get(tag).is_some_and(|b| b.closed_at.is_some()) {
        return Ok(CloseOutcome {
            already: true,
            unhanded: false,
            retired: Vec::new(),
        });
    }
    let pending = request.pending_tests;
    if !pending.is_empty() && !request.force {
        return Err(RuleError::new(format!(
            "{} manual test(s) of {tag} are still to run: {}. Let the owner run them, or pass --force to close anyway.",
            pending.len(),
            pending.join(", ")
        )));
    }
    let rec = ensure_bundle(board, tag, &who.actor, at);
    rec.closed_at = Some(at.to_owned());
    rec.closed_by = Some(who.actor.clone());
    rec.close_note = request.note.map(str::to_owned);
    rec.updated_at = Some(at.to_owned());
    let unhanded = board.handed_build.as_deref() == Some(tag);
    if unhanded {
        board.handed_build = None;
        board.handed_at = None;
    }
    let retired = retire_made_for(board, who, tag, (&format!("bundle {tag} closed"), true), at)?;
    audit(
        board,
        who,
        &close_text(tag, unhanded, pending.len(), &retired, request.note),
        at,
    );
    Ok(CloseOutcome {
        already: false,
        unhanded,
        retired,
    })
}

fn close_text(
    tag: &str,
    unhanded: bool,
    pending: usize,
    retired: &[String],
    note: Option<&str>,
) -> String {
    let mut text = format!("closed bundle {tag}");
    if unhanded {
        text.push_str(" (unhanded)");
    }
    if pending > 0 {
        let _ = write!(text, " with {pending} test(s) still to run");
    }
    if !retired.is_empty() {
        let _ = write!(text, ", retired launcher {}", retired.join(", "));
    }
    if let Some(why) = note {
        let _ = write!(text, ": {why}");
    }
    text
}

/// Undoes a close; the launchers its close retired come back. `None` when the bundle was
/// not closed.
pub fn reopen_bundle(
    board: &mut LauncherBoard,
    who: &Who,
    tag: &str,
    cwd: Option<&str>,
    at: &str,
) -> Result<Option<Vec<String>>, RuleError> {
    let rec = board
        .builds
        .get_mut(tag)
        .ok_or_else(|| RuleError::new(format!("no bundle \"{tag}\".")))?;
    if rec.closed_at.is_none() {
        return Ok(None);
    }
    rec.closed_at = None;
    rec.closed_by = None;
    rec.close_note = None;
    rec.updated_at = Some(at.to_owned());
    let back = restore_made_for(board, who, tag, cwd, at)?;
    let named = if back.is_empty() {
        String::new()
    } else {
        format!(", brought back launcher {}", back.join(", "))
    };
    audit(board, who, &format!("reopened bundle {tag}{named}"), at);
    Ok(Some(back))
}

/// `coord build ready|draft`: ready is built and testable, draft is not built yet. Returns
/// false when the bundle already had that status.
pub fn set_bundle_status(
    board: &mut LauncherBoard,
    who: &Who,
    tag: &str,
    status: BundleStatus,
    at: &str,
) -> Result<bool, RuleError> {
    let rec = board.builds.get(tag).ok_or_else(|| {
        RuleError::new(format!(
            "no bundle \"{tag}\": coord build add {tag} makes it."
        ))
    })?;
    if rec.closed_at.is_some() {
        return Err(RuleError::new(format!(
            "bundle {tag} is closed: coord build reopen {tag} first."
        )));
    }
    if status == BundleStatus::Draft && board.handed_build.as_deref() == Some(tag) {
        return Err(RuleError::new(format!(
            "bundle {tag} is the handed build: coord build unhand first, then coord build draft {tag}."
        )));
    }
    let same = bundle_status(board, tag, Some(rec)) == status;
    if let Some(rec) = board.builds.get_mut(tag) {
        rec.status = Some(status.as_str().to_owned());
        if !same {
            rec.updated_at = Some(at.to_owned());
        }
    }
    if !same {
        audit(
            board,
            who,
            &format!("bundle {tag} is {}", status.as_str()),
            at,
        );
    }
    Ok(!same)
}

/// `coord build hand`: refused for a closed bundle, a draft, and while the project has no
/// master launcher (the owner's own hand from the page is not blocked).
pub fn check_hand(board: &LauncherBoard, tag: &str) -> Result<(), RuleError> {
    let rec = board.builds.get(tag);
    if rec.is_some_and(|b| b.closed_at.is_some()) {
        return Err(RuleError::new(format!(
            "bundle {tag} is closed: coord build reopen {tag} first."
        )));
    }
    if rec.is_some() && bundle_status(board, tag, rec) == BundleStatus::Draft {
        return Err(RuleError::new(format!(
            "bundle {tag} is a draft (not built yet), so it cannot be handed to the owner: coord build ready {tag} once it is built and testable, then hand it."
        )));
    }
    if !has_master(board) {
        return Err(RuleError::new(format!(
            "this project has no master launcher, so nothing can be handed to the owner yet: every project must have one (the main checkout's app). Set master first: coord launcher set {MASTER} --cmd \"<command that starts it>\"."
        )));
    }
    Ok(())
}
