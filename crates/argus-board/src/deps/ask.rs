//! `ask`: open a request in four steps, each under the lock of one board and each safe to run
//! again. The ledger is the truth; a crash between two steps leaves an `opening` dependency
//! that the next reconcile finishes or cancels.
//!
//! 1. portfolio: the checks, then `D-n` in `opening`.
//! 2. assignee: the item with `requestedBy` (made, or found if an earlier try made it).
//! 3. requester: `waitsOn` on the item.
//! 4. portfolio: `to.item` and the state `open`.

use std::path::Path;

use serde_json::{Map, Value, json};

use super::error::DepError;
use super::ledger::{
    COLL, Side, Step, dep_in, deps_in, get, locked, locked_ledger, move_state, set_end,
};
use super::mirror::{put, requested_by, waits_on};
use super::model::{Dep, DepState, Done, Transition};
use super::plan::{AskRequest, Plan, plan};
use super::reconcile::reconcile_with;
use super::roster::Roster;
use super::rules::check_new;
use super::sight::{Seen, find_requested};
use crate::Board;
use crate::json::{doc, nullish};
use crate::store::create_doc;
use crate::text::slug;

/// Open a request from an item of a project to another project, as the contract says.
///
/// # Errors
/// A refusal of the contract (see the messages), or a board that cannot be read or written.
/// A failure after the dependency was made cancels it.
pub fn ask(home: &Path, req: &AskRequest, roster: &dyn Roster) -> Result<Done, DepError> {
    let plan = plan(home, req, roster)?;
    let id = open(home, req, &plan)?;
    match carry_out(home, req, &plan, &id) {
        Ok(moved) => reconcile_with(home, &id, &req.actor, moved),
        Err(e) => {
            give_up(home, &id, &req.actor);
            Err(e)
        }
    }
}

fn open(home: &Path, req: &AskRequest, plan: &Plan) -> Result<String, DepError> {
    locked_ledger(home, |p| {
        let deps = deps_in(p);
        let onto = req.onto.as_deref().map(|o| (req.target.as_str(), o));
        check_new(&deps, (&req.project, &req.item), onto)?;
        let fields = json!({
            "state": "opening",
            "from": {"project": req.project, "item": req.item, "thread": plan.requester_thread},
            "to": {"project": req.target, "item": req.onto, "thread": plan.thread},
            "title": plan.title, "reason": null, "openedAt": null, "resolvedAt": null,
            "history": [],
        });
        Ok(create_doc(p, COLL, "D", object(fields), &req.actor)?)
    })
}

fn carry_out(
    home: &Path,
    req: &AskRequest,
    plan: &Plan,
    id: &str,
) -> Result<Vec<Transition>, DepError> {
    let dep = get(home, id)?;
    let item = request_item(home, &dep, req, plan)?;
    wait_on_it(home, &dep, req, plan)?;
    finish(home, id, &item, &req.actor)
}

/// Step 2: the assignee item that carries `requestedBy`.
fn request_item(home: &Path, dep: &Dep, req: &AskRequest, plan: &Plan) -> Result<String, DepError> {
    locked(home, &req.target, |board| {
        if let Some(found) = find_requested(board, &dep.id) {
            return Ok(found);
        }
        let mirror = requested_by(dep, &plan.requester_title);
        match req.onto.as_deref() {
            Some(onto) => attach(board, onto, mirror, &req.target),
            None => make(board, (req, plan), mirror),
        }
    })
}

fn attach(board: &mut Board, onto: &str, mirror: Value, target: &str) -> Result<String, DepError> {
    let open = doc(board, "items", onto).is_some_and(|i| Seen::of(onto, i).open());
    if !open {
        return Err(DepError::Unknown(format!(
            "no open item {onto} on {target}"
        )));
    }
    put(board, onto, Side::Assignee, mirror);
    Ok(onto.to_owned())
}

fn object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn make(
    board: &mut Board,
    (req, plan): (&AskRequest, &Plan),
    mirror: Value,
) -> Result<String, DepError> {
    let subject = doc(board, "threads", &plan.thread)
        .and_then(|t| nullish(t, "subject"))
        .unwrap_or_else(|| "general".to_owned());
    let ac: Vec<Value> = plan
        .ac
        .iter()
        .zip(1_u32..)
        .map(|(text, n)| json!({"n": n, "text": text, "done": false}))
        .collect();
    let fields = json!({
        "title": plan.title, "description": plan.description, "subject": slug(&subject),
        "thread": plan.thread, "status": "todo", "origin": "coordinator", "ac": ac,
        "requestedBy": mirror,
    });
    Ok(create_doc(board, "items", "I", object(fields), &req.actor)?)
}

/// Step 3: `waitsOn` on the requester item.
fn wait_on_it(home: &Path, dep: &Dep, req: &AskRequest, plan: &Plan) -> Result<(), DepError> {
    locked(home, &req.project, |board| {
        if doc(board, "items", &req.item).is_none() {
            return Err(DepError::Unknown(format!(
                "no item {} on {}",
                req.item, req.project
            )));
        }
        put(
            board,
            &req.item,
            Side::Requester,
            waits_on(dep, &plan.assignee_title),
        );
        Ok(())
    })
}

/// Step 4: the assignee item into the ledger and the state `open`.
fn finish(home: &Path, id: &str, item: &str, by: &str) -> Result<Vec<Transition>, DepError> {
    locked_ledger(home, |p| {
        set_end(p, id, (Side::Assignee, "item", item), by)?;
        let opening = dep_in(p, id).is_some_and(|d| d.state == DepState::Opening);
        if !opening {
            return Ok(Vec::new());
        }
        let step = Step {
            to: DepState::Open,
            by,
            reason: None,
        };
        Ok(vec![move_state(p, id, &step)?])
    })
}

/// A request that could not be carried out is cancelled, and its new item dropped.
fn give_up(home: &Path, id: &str, by: &str) {
    let step = Step {
        to: DepState::Cancelled,
        by,
        reason: Some("the request could not be opened"),
    };
    let moved = locked_ledger(home, |p| Ok(vec![move_state(p, id, &step)?]));
    if let Ok(moved) = moved {
        reconcile_with(home, id, by, moved).ok();
    }
}
