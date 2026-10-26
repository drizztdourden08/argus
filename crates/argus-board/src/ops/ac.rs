//! The `ac` op: an item's acceptance criteria. They are numbered by position and tests point at
//! them by number, so an add, rm or move renumbers the list and each test's covered numbers
//! follow.

use serde_json::{Map, Value, json};

use super::{
    announce::announce,
    authorize::{Check, authorize},
    result::{OpResult, version_check},
    target::target_of,
    who::Who,
};
use crate::{
    Board,
    error::{BoardError, refuse},
    json::{array_mut, as_integer, doc, doc_mut, find_ids, same, set, shown},
    limits::require_text,
    store::stamp,
};

const AC_VERBS: [&str; 6] = ["add", "set", "done", "undo", "rm", "move"];

/// How the numbers a test covers move after the list changed.
enum Renumber {
    /// A criterion was inserted at this position: later ones shift up.
    Insert(i64),
    /// Positions from..=last were removed: later ones shift down, removed ones vanish.
    Remove { from: i64, last: i64 },
    /// The list was reordered: the new order of the old numbers.
    Reorder(Vec<i64>),
}

impl Renumber {
    fn apply(&self, k: i64) -> Option<i64> {
        match self {
            Renumber::Insert(at) => Some(if k >= *at { k + 1 } else { k }),
            Renumber::Remove { from, last } => {
                if k < *from {
                    Some(k)
                } else if k > *last {
                    Some(k - (last - from + 1))
                } else {
                    None
                }
            }
            Renumber::Reorder(order) => {
                let at = order.iter().position(|o| *o == k);
                at.and_then(|i| i64::try_from(i).ok()).map(|i| i + 1)
            }
        }
    }
}

/// What a verb did to the list.
struct Change {
    outcome: String,
    renumber: Option<Renumber>,
}

fn number(v: Option<&Value>, len: usize, id: &str) -> Result<i64, BoardError> {
    let in_range =
        as_integer(v).filter(|n| usize::try_from(*n).is_ok_and(|n| (1..=len).contains(&n)));
    in_range.ok_or_else(|| refuse(format!("{id} has no AC{}.", shown(v))))
}

fn position(v: Option<&Value>, max: usize) -> Result<i64, BoardError> {
    let in_range =
        as_integer(v).filter(|n| usize::try_from(*n).is_ok_and(|n| (1..=max).contains(&n)));
    in_range.ok_or_else(|| refuse(format!("position {} is not within 1-{max}.", shown(v))))
}

fn index(n: i64) -> usize {
    usize::try_from(n - 1).unwrap_or(0)
}

fn do_add(
    ac: &mut Vec<Value>,
    op: &Value,
    line: Option<&str>,
    id: &str,
) -> Result<Change, BoardError> {
    let len = ac.len();
    let at = match op.get("to") {
        None => i64::try_from(len + 1).unwrap_or(i64::MAX),
        to => position(to, len + 1)?,
    };
    ac.insert(
        index(at).min(len),
        json!({"n": at, "text": line, "done": false}),
    );
    Ok(Change {
        outcome: format!("{id} AC{at} added"),
        renumber: Some(Renumber::Insert(at)),
    })
}

fn do_rm(ac: &mut Vec<Value>, op: &Value, len: usize, id: &str) -> Result<Change, BoardError> {
    let from = number(op.get("n"), len, id)?;
    let last = number(
        Some(op.get("m").filter(|m| !m.is_null()).unwrap_or(&op["n"])),
        len,
        id,
    )?;
    if last < from {
        return Err(refuse(format!("{last} is before {from}.")));
    }
    ac.drain(index(from)..=index(last));
    let range = if from == last {
        from.to_string()
    } else {
        format!("{from}-{last}")
    };
    Ok(Change {
        outcome: format!("{id} AC{range} removed"),
        renumber: Some(Renumber::Remove { from, last }),
    })
}

fn do_move(ac: &mut Vec<Value>, op: &Value, len: usize, id: &str) -> Result<Change, BoardError> {
    let from = number(op.get("n"), len, id)?;
    let at = position(op.get("to"), len)?;
    let mut order: Vec<i64> = (1_i64..).take(len).collect();
    order.remove(index(from));
    order.insert(index(at), from);
    *ac = order.iter().map(|k| ac[index(*k)].clone()).collect();
    Ok(Change {
        outcome: format!("{id} AC{from} moved to {at}"),
        renumber: Some(Renumber::Reorder(order)),
    })
}

fn do_edit(
    ac: &mut [Value],
    (verb, op): (&str, &Value),
    line: Option<&str>,
    (len, id): (usize, &str),
) -> Result<Change, BoardError> {
    let n = number(op.get("n"), len, id)?;
    let shown_n = shown(op.get("n"));
    let criterion = &mut ac[index(n)];
    let outcome = if verb == "set" {
        set(criterion, "text", line);
        format!("{id} AC{shown_n} edited")
    } else {
        set(criterion, "done", verb == "done");
        let word = if verb == "done" { "done" } else { "reopened" };
        format!("{id} AC{shown_n} {word}")
    };
    Ok(Change {
        outcome,
        renumber: None,
    })
}

fn change_list(
    ac: &mut Vec<Value>,
    (verb, op): (&str, &Value),
    line: Option<&str>,
    id: &str,
) -> Result<Change, BoardError> {
    let len = ac.len();
    match verb {
        "add" => do_add(ac, op, line, id),
        "rm" => do_rm(ac, op, len, id),
        "move" => do_move(ac, op, len, id),
        _ => do_edit(ac, (verb, op), line, (len, id)),
    }
}

fn renumber_tests(
    board: &mut Board,
    item_id: &str,
    renumber: &Renumber,
    actor: &str,
) -> Result<(), BoardError> {
    let covering = |t: &Value| {
        t["item"].as_str() == Some(item_id) && t["ac"].as_array().is_some_and(|a| !a.is_empty())
    };
    for id in find_ids(board, "tests", covering) {
        let old = doc(board, "tests", &id).map_or(Value::Null, |t| t["ac"].clone());
        let mut next: Vec<i64> = Vec::new();
        for k in old
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_i64)
        {
            if let Some(n) = renumber.apply(k).filter(|n| !next.contains(n)) {
                next.push(n);
            }
        }
        next.sort_unstable();
        if same(Some(&json!(next)), Some(&old)) {
            continue;
        }
        if let Some(t) = doc_mut(board, "tests", &id) {
            set(t, "ac", json!(next));
        }
        stamp(board, "tests", &id, actor)?;
    }
    Ok(())
}

fn conflict_check(
    item: &Value,
    op: &Value,
    verb: &str,
    line: Option<&str>,
) -> Option<Map<String, Value>> {
    if verb == "add" {
        return None;
    }
    let slot = as_integer(op.get("n")).and_then(|n| usize::try_from(n - 1).ok());
    let text = slot
        .and_then(|i| item["ac"].get(i))
        .map_or(Value::Null, |c| c["text"].clone());
    let seen = json!({"rev": item["rev"], "text": text});
    let next = Map::from_iter([("text".to_owned(), json!(line))]);
    version_check(&seen, op, &["text".to_owned()], &next)
}

/// Apply an `ac` op.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let (coll, id) = target_of(board, op, false)?;
    if coll != "items" {
        return Err(refuse("only an item has acceptance criteria."));
    }
    let item = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    authorize(who, &Check::new("ac", &coll, Some(&item)))?;
    let verb = op["do"].as_str().filter(|v| AC_VERBS.contains(v));
    let Some(verb) = verb else {
        return Err(refuse(format!(
            "ac op must be one of {}.",
            AC_VERBS.join(", ")
        )));
    };
    let line = if matches!(verb, "add" | "set") {
        Some(require_text("ac", &op["text"])?)
    } else {
        None
    };
    if let Some(collision) = conflict_check(&item, op, verb, line.as_deref()) {
        return Ok(OpResult::conflict(&item, collision));
    }
    let Some(target) = doc_mut(board, &coll, &id) else {
        return Err(refuse("gone"));
    };
    let change = change_list(array_mut(target, "ac"), (verb, op), line.as_deref(), &id)?;
    for (i, criterion) in array_mut(target, "ac").iter_mut().enumerate() {
        set(criterion, "n", i + 1);
    }
    let actor = who.actor_of();
    if let Some(renumber) = &change.renumber {
        renumber_tests(board, &id, renumber, &actor)?;
    }
    stamp(board, "items", &id, &actor)?;
    let said = change
        .outcome
        .replacen(&format!("{id} "), &format!("changed {id}: "), 1);
    let owner = item["thread"].as_str().map(str::to_owned);
    let notify = announce(board, who, &[owner], &said, Some(&id));
    Ok(OpResult::applied(change.outcome).with_notify(notify))
}
