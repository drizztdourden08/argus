//! The `move` op: an item or action changing hands, an item changing status, a test or entry
//! attaching to another item, or any record taking a new place in the order.

use argus_core::js;
use serde_json::{Map, Value, json};

use super::announce::{announce, refresh_needs_human};
use super::authorize::{Check, authorize};
use super::result::{OpResult, version_check};
use super::target::{
    ITEM_STATUSES, id_arg, item_arg, one_of, record_arg, record_thread, target_of, thread_arg,
};
use super::typed::{self, Outcome};
use super::who::Who;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{as_integer, display, doc, doc_mut, find_ids, nullish, obj, set, strict_eq};
use crate::store::{rank_between, stamp, stamp_lane};

const OPEN: [&str; 3] = ["open", "pending", "doing"];

/// What a move changes: the fields, and for a test the acceptance criteria it now covers.
struct Plan {
    changes: Map<String, Value>,
    ac_refs: Option<Vec<Value>>,
}

impl Plan {
    fn of(changes: Map<String, Value>) -> Self {
        Plan {
            changes,
            ac_refs: None,
        }
    }
}

/// The thread a move names, checked, as a change.
fn thread_change(board: &Board, to: &Map<String, Value>) -> Result<Map<String, Value>, BoardError> {
    let mut changes = Map::new();
    if let Some(thread) = to.get("thread") {
        changes.insert("thread".into(), Value::String(thread_arg(board, thread)?));
    }
    Ok(changes)
}

fn plan_item(board: &Board, to: &Map<String, Value>) -> Result<Plan, BoardError> {
    if to.contains_key("item") {
        return Err(refuse("an item cannot move into another item."));
    }
    let mut changes = thread_change(board, to)?;
    if let Some(status) = to.get("status") {
        changes.insert("status".into(), one_of(&ITEM_STATUSES, status, "status")?);
    }
    Ok(Plan::of(changes))
}

fn ac_refs(item: &Value, item_id: &str, ac: Option<&Value>) -> Result<Vec<Value>, BoardError> {
    let list = match ac {
        None | Some(Value::Null) => &[][..],
        Some(Value::Array(list)) => list.as_slice(),
        Some(_) => return Err(refuse("ac must be a list.")),
    };
    let len = item["ac"].as_array().map_or(0, Vec::len);
    list.iter()
        .map(|v| {
            let n = as_integer(Some(v)).and_then(|n| usize::try_from(n).ok());
            match n.filter(|n| (1..=len).contains(n)) {
                Some(n) => Ok(json!(n)),
                None => Err(refuse(format!("{item_id} has no AC{}.", display(v)))),
            }
        })
        .collect()
}

fn plan_test(board: &Board, to: &Map<String, Value>) -> Result<Plan, BoardError> {
    if to.contains_key("status") {
        return Err(refuse(
            "a test's status is a result; record one with coord test result.",
        ));
    }
    if to.contains_key("thread") {
        return Err(refuse(
            "a test follows its item to the item's thread; move it to an item instead.",
        ));
    }
    let Some(target) = to.get("item") else {
        return Ok(Plan::of(Map::new()));
    };
    let item_id = item_arg(board, target)?;
    let item = doc(board, "items", &item_id)
        .cloned()
        .unwrap_or(Value::Null);
    let mut changes = Map::new();
    changes.insert("item".into(), json!(item_id));
    changes.insert("thread".into(), item["thread"].clone());
    if !item["subject"].is_null() {
        changes.insert("subject".into(), item["subject"].clone());
    }
    let refs = ac_refs(&item, &item_id, to.get("ac"))?;
    Ok(Plan {
        changes,
        ac_refs: Some(refs),
    })
}

fn plan_action(board: &Board, to: &Map<String, Value>) -> Result<Plan, BoardError> {
    if to.contains_key("status") || to.contains_key("item") {
        return Err(refuse("use edit to change an action's status or item."));
    }
    Ok(Plan::of(thread_change(board, to)?))
}

fn plan_entry(board: &Board, to: &Map<String, Value>) -> Result<Plan, BoardError> {
    if to.contains_key("thread") || to.contains_key("status") {
        return Err(refuse(
            "an entry follows its item's thread; move it to an item instead.",
        ));
    }
    let mut changes = Map::new();
    if let Some(target) = to.get("item") {
        let record = record_arg(board, target)?;
        let thread = record_thread(board, &record);
        changes.insert("item".into(), json!(record));
        changes.insert("thread".into(), thread);
    }
    Ok(Plan::of(changes))
}

fn plan_order_only(coll: &str, to: &Map<String, Value>) -> Result<Plan, BoardError> {
    if ["thread", "status", "item"]
        .iter()
        .any(|k| to.contains_key(*k))
    {
        let what = if coll == "agents" {
            "an agent"
        } else {
            "a thread"
        };
        return Err(refuse(format!(
            "{what} can only change its place in the order."
        )));
    }
    Ok(Plan::of(Map::new()))
}

fn plan(board: &Board, coll: &str, to: &Map<String, Value>) -> Result<Plan, BoardError> {
    match coll {
        "items" => plan_item(board, to),
        "tests" => plan_test(board, to),
        "actions" => plan_action(board, to),
        "entries" => plan_entry(board, to),
        _ => plan_order_only(coll, to),
    }
}

/// An item changing hands: its tests follow, and so does every open question, decision and
/// notice routed to the old owner on it.
pub(super) fn hand_over(
    board: &mut Board,
    item_id: &str,
    from: &Value,
    actor: &str,
) -> Result<(), BoardError> {
    let thread = doc(board, "items", item_id).map_or(Value::Null, |i| i["thread"].clone());
    for id in find_ids(board, "tests", |t| t["item"].as_str() == Some(item_id)) {
        if let Some(t) = doc_mut(board, "tests", &id) {
            set(t, "thread", thread.clone());
        }
        stamp(board, "tests", &id, actor)?;
    }
    let open = |e: &Value| {
        e["item"].as_str() == Some(item_id)
            && strict_eq(&e["thread"], from)
            && e["status"].as_str().is_some_and(|s| OPEN.contains(&s))
    };
    for id in find_ids(board, "entries", open) {
        if let Some(e) = doc_mut(board, "entries", &id) {
            set(e, "thread", thread.clone());
        }
        stamp(board, "entries", &id, actor)?;
    }
    Ok(())
}

fn parts_of(coll: &str, changes: &Map<String, Value>, placed: bool) -> String {
    let mut parts = Vec::new();
    let shown = |key: &str| changes.get(key).filter(|v| js::truthy(v)).map(display);
    if let Some(thread) = shown("thread").filter(|_| coll != "tests") {
        parts.push(format!("to {thread}"));
    }
    if let Some(item) = shown("item") {
        parts.push(format!("to {item}"));
    }
    if let Some(status) = shown("status") {
        parts.push(format!("(status {status})"));
    }
    if placed {
        parts.push("in the order".to_owned());
    }
    parts.join(" ")
}

fn write_move(board: &mut Board, (coll, id): (&str, &str), plan: &Plan, rank: Option<f64>) {
    let Some(d) = doc_mut(board, coll, id) else {
        return;
    };
    let was = d["status"].clone();
    for (field, value) in &plan.changes {
        set(d, field, value.clone());
    }
    if coll == "items" {
        stamp_lane(d, &was);
    }
    if let Some(refs) = &plan.ac_refs {
        set(d, "ac", Value::Array(refs.clone()));
    }
    if let Some(rank) = rank {
        set(d, "rank", js::number(rank));
    }
}

fn after_move(
    board: &mut Board,
    (coll, id): (&str, &str),
    plan: &Plan,
    was: &Value,
    actor: &str,
) -> Result<(), BoardError> {
    let moved_thread = plan.changes.get("thread").filter(|t| js::truthy(t));
    if coll == "items" && moved_thread.is_some_and(|t| !strict_eq(t, was)) {
        hand_over(board, id, was, actor)?;
    }
    if coll == "actions" && moved_thread.is_some() {
        let now = doc(board, coll, id)
            .map(|d| display(&d["thread"]))
            .unwrap_or_default();
        refresh_needs_human(board, &display(was));
        refresh_needs_human(board, &now);
    }
    Ok(())
}

/// A typed item changing stage: the status and stage to write, and what the move let through.
fn shifted(
    board: &Board,
    (coll, current): (&str, &Value),
    by: (&Who, &Value),
    plan: &mut Plan,
    to: &Map<String, Value>,
) -> Result<Outcome, BoardError> {
    if coll != "items" {
        return Ok(Outcome::default());
    }
    typed::on_move(board, current, by, &mut plan.changes, to)
}

/// Apply a `move` op.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let (coll, id) = target_of(board, op, false)?;
    let to = obj(&op["to"]);
    let mut plan = plan(board, &coll, to)?;
    let before = to.get("before").and_then(id_arg);
    let after = to.get("after").and_then(id_arg);
    let placed = before.is_some() || after.is_some();
    let staged = coll == "items" && to.get("stage").is_some_and(|s| !s.is_null());
    if plan.changes.is_empty() && !placed && !staged {
        return Err(refuse("nothing to move."));
    }
    let current = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    let new_thread = plan.changes.get("thread").and_then(Value::as_str);
    authorize(
        who,
        &Check::new("move", &coll, Some(&current)).to(new_thread),
    )?;
    let outcome = shifted(board, (&coll, &current), (who, op), &mut plan, to)?;
    let rank = if placed {
        Some(rank_between(
            board,
            &coll,
            before.as_deref(),
            after.as_deref(),
            Some(&id),
        )?)
    } else {
        None
    };
    let named: Vec<String> = ["thread", "status", "item", "stage"]
        .iter()
        .filter(|k| to.contains_key(**k))
        .map(|k| (*k).to_owned())
        .collect();
    if let Some(collision) = version_check(&current, op, &named, &plan.changes) {
        return Ok(OpResult::conflict(&current, collision));
    }
    let actor = who.actor_of();
    write_move(board, (&coll, &id), &plan, rank);
    stamp(board, &coll, &id, &actor)?;
    after_move(board, (&coll, &id), &plan, &current["thread"], &actor)?;
    outcome.record(board, &id, who)?;
    let now = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    let owners = if coll == "threads" {
        vec![Some(id.clone())]
    } else {
        vec![nullish(&current, "thread"), nullish(&now, "thread")]
    };
    let parts = parts_of(&coll, &plan.changes, placed);
    let said = format!("moved {id} {parts}").trim().to_owned();
    let notify = announce(board, who, &owners, &said, Some(&id));
    Ok(outcome.finish(OpResult::applied(said).with_notify(notify)))
}
