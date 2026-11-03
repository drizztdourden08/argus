//! Reconcile: read both items, decide the transition of the state machine, apply it to the
//! ledger under the portfolio lock with a check on the revision, then rewrite each mirror that
//! differs. Each board is locked on its own, never two at once.

use std::path::Path;

use argus_core::clock::{now_ms, parse_ms};

use super::decide::{Verdict, decide};
use super::error::DepError;
use super::ledger::{self, Side, Step, dep_in, deps_in, locked_ledger, move_state, set_end};
use super::mirror::settle;
use super::model::{Dep, DepState, Done, Transition};
use super::sight::{Look, Sight};
use crate::Board;

const ARGUS: &str = "argus";
/// A done dependency is watched for a reopen this long after it was resolved.
const REOPEN_WINDOW_MS: i64 = 14 * 24 * 3_600_000;

/// What a sweep did: the transitions made and the dependencies it could not reconcile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sweep {
    /// Every transition made, with the dependency order of the ledger.
    pub transitions: Vec<Transition>,
    /// `(dependency id, message)` of each one that failed; the next sweep tries again.
    pub failed: Vec<(String, String)>,
}

/// What is to change in the ledger for one dependency.
struct Change {
    threads: Vec<(Side, String)>,
    item: Option<String>,
    verdict: Verdict,
}

impl Change {
    fn is_none(&self) -> bool {
        self.threads.is_empty() && self.item.is_none() && self.verdict == Verdict::Stay
    }
}

fn thread_moves(dep: &Dep, sight: &Sight) -> Vec<(Side, String)> {
    let mut out = Vec::new();
    for (side, look, known) in [
        (Side::Requester, &sight.requester, &dep.from.thread),
        (Side::Assignee, &sight.assignee, &dep.to.thread),
    ] {
        let seen = look.item().and_then(|i| i.thread.clone());
        if let Some(thread) = seen.filter(|t| known.as_deref() != Some(t.as_str())) {
            out.push((side, thread));
        }
    }
    out
}

fn change_for(dep: &Dep, sight: &Sight) -> Change {
    let watched = dep.state.is_live() || dep.state == DepState::Done;
    let found = match &sight.assignee {
        Look::Item(i) if dep.to.item.is_none() => Some(i.id.clone()),
        _ => None,
    };
    Change {
        threads: if watched {
            thread_moves(dep, sight)
        } else {
            Vec::new()
        },
        item: found,
        verdict: decide(dep, sight),
    }
}

fn write_change(
    ledger: &mut Board,
    dep: &Dep,
    change: &Change,
    by: &str,
) -> Result<(Dep, Vec<Transition>), DepError> {
    let Some(now) = dep_in(ledger, &dep.id) else {
        return Err(ledger::unknown(&dep.id));
    };
    if now.rev != dep.rev {
        return Ok((now, Vec::new()));
    }
    for (side, thread) in &change.threads {
        set_end(ledger, &dep.id, (*side, "thread", thread), by)?;
    }
    if let Some(item) = &change.item {
        set_end(ledger, &dep.id, (Side::Assignee, "item", item), by)?;
    }
    let mut moved = Vec::new();
    if let Verdict::Move(to, why) = &change.verdict {
        let step = Step {
            to: *to,
            by,
            reason: why.as_deref(),
        };
        moved.push(move_state(ledger, &dep.id, &step)?);
    }
    let after = dep_in(ledger, &dep.id).unwrap_or(now);
    Ok((after, moved))
}

/// Reconcile one dependency as `by`, with the transitions the caller already made in `earlier`
/// (they are told to the boards with the new ones).
pub(crate) fn reconcile_with(
    home: &Path,
    id: &str,
    by: &str,
    mut earlier: Vec<Transition>,
) -> Result<Done, DepError> {
    let board = ledger::read(home)?;
    let deps = deps_in(&board);
    let Some(dep) = deps.iter().find(|d| d.id == id).cloned() else {
        return Err(ledger::unknown(id));
    };
    let sight = Sight::read(home, &board, &deps, &dep)?;
    let change = change_for(&dep, &sight);
    let after = if change.is_none() {
        dep
    } else {
        let (after, moved) = locked_ledger(home, |p| write_change(p, &dep, &change, by))?;
        earlier.extend(moved);
        after
    };
    settle(home, &after, &sight, &earlier)?;
    Ok(Done::of(after, earlier))
}

/// Reconcile one dependency: read both items, move it along the state machine when they say so,
/// and repair both mirrors. Answers the transitions made.
///
/// # Errors
/// The dependency is unknown, or a board cannot be read, locked or written.
pub fn reconcile(home: &Path, id: &str) -> Result<Vec<Transition>, DepError> {
    reconcile_with(home, id, ARGUS, Vec::new()).map(|done| done.transitions)
}

fn watched(dep: &Dep, now: i64) -> bool {
    match dep.state {
        DepState::Done => dep
            .resolved_at
            .as_deref()
            .and_then(parse_ms)
            .is_none_or(|at| now - at < REOPEN_WINDOW_MS),
        state => state.is_live(),
    }
}

fn run(home: &Path, ids: impl IntoIterator<Item = String>) -> Sweep {
    let mut out = Sweep::default();
    for id in ids {
        match reconcile(home, &id) {
            Ok(moved) => out.transitions.extend(moved),
            Err(e) => out.failed.push((id, e.to_string())),
        }
    }
    out
}

/// Reconcile every dependency that is not over: the live ones, and the done ones that could
/// still be reopened. The server runs it every 5 minutes.
///
/// # Errors
/// The ledger cannot be read.
pub fn sweep(home: &Path) -> Result<Sweep, DepError> {
    let now = now_ms();
    let ids = deps_in(&ledger::read(home)?)
        .into_iter()
        .filter(|d| watched(d, now))
        .map(|d| d.id);
    Ok(run(home, ids))
}

/// Reconcile the dependencies that have an end on a project: the server runs it when that
/// board was exported, whoever wrote it.
///
/// # Errors
/// The ledger cannot be read.
pub fn reconcile_board(home: &Path, project: &str) -> Result<Sweep, DepError> {
    let now = now_ms();
    let ids = deps_in(&ledger::read(home)?)
        .into_iter()
        .filter(|d| watched(d, now) && (d.from.project == project || d.to.project == project))
        .map(|d| d.id);
    Ok(run(home, ids))
}
