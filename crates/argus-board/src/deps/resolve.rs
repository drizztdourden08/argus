//! The verbs on a dependency that is already open: `close` (as done, dropped or blocked),
//! `reopen` and `cancel`. Each checks who calls and the state, moves the ledger or the assignee
//! item, then reconciles so both mirrors and both boards follow.

use std::path::Path;

use serde::Deserialize;
use serde_json::json;

use super::cycle::closes;
use super::error::{
    DepError, bad, blocked_needs_reason, closes_loop, not_a_dependency_of, not_open, wrong_state,
};
use super::ledger::{Side, Step, dep_in, deps_in, get, locked, locked_ledger, move_state, read};
use super::model::{Dep, DepState, Done, Transition, project_of};
use super::reconcile::reconcile_with;
use super::sight::find_requested;
use crate::json::{doc, doc_mut, truthy};
use crate::limits::optional_text;
use crate::store::{set_lane, stamp};

/// The body of `POST /api/deps/:id/close`.
#[derive(Debug, Clone, Deserialize)]
pub struct CloseRequest {
    /// `done`, `dropped` or `blocked`.
    #[serde(rename = "as")]
    pub how: String,
    /// Why; `blocked` needs one.
    #[serde(default)]
    pub reason: Option<String>,
    /// Who closes.
    pub actor: String,
}

/// The body of `POST /api/deps/:id/reopen`.
#[derive(Debug, Clone, Deserialize)]
pub struct ReopenRequest {
    /// Who reopens.
    pub actor: String,
}

/// The body of `POST /api/deps/:id/cancel`.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelRequest {
    /// Why.
    #[serde(default)]
    pub reason: Option<String>,
    /// Who cancels.
    pub actor: String,
}

/// The states a verb accepts.
type Needs = (&'static str, [DepState; 2]);

const CLOSE: Needs = ("close", [DepState::Open, DepState::Blocked]);
const REOPEN: Needs = ("reopen", [DepState::Blocked, DepState::Done]);
const CANCEL: Needs = ("cancel", [DepState::Open, DepState::Blocked]);

fn need(dep: &Dep, (verb, allowed): Needs) -> Result<(), DepError> {
    if allowed.contains(&dep.state) {
        Ok(())
    } else {
        Err(wrong_state(
            &dep.id,
            dep.state.as_str(),
            verb,
            [allowed[0].as_str(), allowed[1].as_str()],
        ))
    }
}

/// Only the project on that side (or the owner) may call.
fn side_check(dep: &Dep, side: Side, actor: &str) -> Result<(), DepError> {
    let project = match side {
        Side::Requester => &dep.from.project,
        Side::Assignee => &dep.to.project,
    };
    match project_of(actor) {
        Some(caller) if caller != project => Err(not_a_dependency_of(&dep.id, caller)),
        _ => Ok(()),
    }
}

fn reason_of(reason: Option<&str>) -> Result<Option<String>, DepError> {
    optional_text("reason", &json!(reason)).map_err(bad)
}

/// Move the ledger, with the state checked again under the lock.
fn transit(home: &Path, id: &str, step: &Step, needs: Needs) -> Result<Vec<Transition>, DepError> {
    locked_ledger(home, |p| {
        let Some(dep) = dep_in(p, id) else {
            return Err(super::ledger::unknown(id));
        };
        need(&dep, needs)?;
        Ok(vec![move_state(p, id, step)?])
    })
}

/// Set the status of the assignee item, when it is there, not removed, and (if `only_from` is
/// named) in that status.
fn set_status(
    home: &Path,
    dep: &Dep,
    (only_from, to): (Option<&str>, &str),
    by: &str,
) -> Result<(), DepError> {
    locked(home, &dep.to.project, |board| {
        let id = dep
            .to
            .item
            .clone()
            .filter(|i| doc(board, "items", i).is_some())
            .or_else(|| find_requested(board, &dep.id));
        let Some(id) = id else { return Ok(()) };
        let Some(item) = doc_mut(board, "items", &id) else {
            return Ok(());
        };
        let removed = truthy(item, "deletedAt") || truthy(item, "archivedAt");
        let from_ok = only_from.is_none_or(|f| item["status"] == f);
        if removed || !from_ok || item["status"] == to {
            return Ok(());
        }
        set_lane(item, to);
        stamp(board, "items", &id, by)?;
        Ok(())
    })
}

/// Close a request as the assignee: `done` and `dropped` set the item status and the
/// dependency follows; `blocked` needs a reason and moves the item out of `doing`.
///
/// # Errors
/// A refusal of the contract, or a board that cannot be read or written.
pub fn close(home: &Path, id: &str, req: &CloseRequest) -> Result<Done, DepError> {
    let dep = get(home, id)?;
    side_check(&dep, Side::Assignee, &req.actor)?;
    let reason = reason_of(req.reason.as_deref())?;
    if req.how == "blocked" && reason.is_none() {
        return Err(blocked_needs_reason());
    }
    if !["done", "dropped", "blocked"].contains(&req.how.as_str()) {
        return Err(DepError::Bad(
            "--as must be done, dropped or blocked".to_owned(),
        ));
    }
    need(&dep, CLOSE)?;
    match reason {
        Some(why) if req.how == "blocked" => block(home, &dep, &why, &req.actor),
        _ => {
            set_status(home, &dep, (None, &req.how), &req.actor)?;
            reconcile_with(home, id, &req.actor, Vec::new())
        }
    }
}

fn block(home: &Path, dep: &Dep, why: &str, by: &str) -> Result<Done, DepError> {
    set_status(home, dep, (Some("doing"), "todo"), by)?;
    let step = Step {
        to: DepState::Blocked,
        by,
        reason: Some(why),
    };
    let moved = transit(home, &dep.id, &step, CLOSE)?;
    reconcile_with(home, &dep.id, by, moved)
}

/// Reopen a request as the assignee: `blocked` goes back to `open`; `done` puts the item back
/// to `doing` and comes back to `open` when its link is still open and no loop forms.
///
/// # Errors
/// A refusal of the contract, or a board that cannot be read or written.
pub fn reopen(home: &Path, id: &str, req: &ReopenRequest) -> Result<Done, DepError> {
    let dep = get(home, id)?;
    side_check(&dep, Side::Assignee, &req.actor)?;
    need(&dep, REOPEN)?;
    if dep.state == DepState::Blocked {
        let step = Step {
            to: DepState::Open,
            by: &req.actor,
            reason: None,
        };
        let moved = transit(home, id, &step, REOPEN)?;
        return reconcile_with(home, id, &req.actor, moved);
    }
    set_status(home, &dep, (Some("done"), "doing"), &req.actor)?;
    let done = reconcile_with(home, id, &req.actor, Vec::new())?;
    if done.dep.state == DepState::Open {
        Ok(done)
    } else {
        Err(stays_done(home, &dep)?)
    }
}

/// Why a done request did not come back.
fn stays_done(home: &Path, dep: &Dep) -> Result<DepError, DepError> {
    let deps = deps_in(&read(home)?);
    Ok(match closes(&deps, &dep.requester(), &dep.assignee()) {
        Some(path) => closes_loop(&path),
        None => not_open(&dep.requester(), "no longer waiting"),
    })
}

/// Cancel a request as the requester (or the owner): the assignee is told, and an item nobody
/// started is dropped.
///
/// # Errors
/// A refusal of the contract, or a board that cannot be read or written.
pub fn cancel(home: &Path, id: &str, req: &CancelRequest) -> Result<Done, DepError> {
    let dep = get(home, id)?;
    side_check(&dep, Side::Requester, &req.actor)?;
    need(&dep, CANCEL)?;
    let reason = reason_of(req.reason.as_deref())?;
    let step = Step {
        to: DepState::Cancelled,
        by: &req.actor,
        reason: reason.as_deref(),
    };
    let moved = transit(home, id, &step, CANCEL)?;
    reconcile_with(home, id, &req.actor, moved)
}
