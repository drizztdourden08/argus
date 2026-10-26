//! Manual order. A doc's rank is a number; lists sort by it. A new doc goes to the end, and a doc
//! dropped between two others takes the midpoint of their ranks.

use std::cmp::Ordering;

use argus_core::js;
use serde_json::Value;

use super::docs::touch;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc_mut, section};

const RANK_STEP: f64 = 1024.0;
const RANK_GAP: f64 = 1e-6;

/// The rank of a doc, 0 when it has none.
fn rank_of(doc: &Value) -> f64 {
    doc["rank"].as_f64().unwrap_or(0.0)
}

fn created(doc: &Value) -> String {
    match &doc["createdAt"] {
        Value::Null => String::new(),
        v => display(v),
    }
}

/// Order by rank, then by creation time.
fn by_rank(a: &Value, b: &Value) -> Ordering {
    match rank_of(a).partial_cmp(&rank_of(b)) {
        Some(order) if order != Ordering::Equal => order,
        _ => created(a).cmp(&created(b)),
    }
}

fn position_f64(i: usize) -> f64 {
    f64::from(u32::try_from(i).unwrap_or(u32::MAX))
}

/// The rank that puts a new doc at the end of a collection.
#[must_use]
pub fn rank_end(board: &Board, coll: &str) -> f64 {
    section(board, coll)
        .values()
        .fold(0.0_f64, |max, doc| max.max(rank_of(doc)))
        + RANK_STEP
}

fn renumber(board: &mut Board, coll: &str) {
    let mut order: Vec<(String, Value)> = section(board, coll)
        .iter()
        .map(|(id, d)| (id.clone(), d.clone()))
        .collect();
    order.sort_by(|a, b| by_rank(&a.1, &b.1));
    for (i, (id, _)) in order.iter().enumerate() {
        if let Some(doc) = doc_mut(board, coll, id) {
            crate::json::set(doc, "rank", js::number((position_f64(i) + 1.0) * RANK_STEP));
        }
        touch(board, coll, id);
    }
}

fn index_of(sorted: &[&Value], id: &str) -> Option<usize> {
    sorted.iter().position(|d| d["id"].as_str() == Some(id))
}

fn bounds(
    board: &Board,
    coll: &str,
    sorted: &[&Value],
    (before, after): (Option<&str>, Option<&str>),
) -> (Option<f64>, Option<f64>) {
    let at_rank = |i: Option<usize>| i.and_then(|i| sorted.get(i)).map(|d| rank_of(d));
    if let Some(a) = after {
        let at = index_of(sorted, a);
        let high = match before {
            Some(b) => section(board, coll).get(b).map(rank_of),
            None => at_rank(at.map(|i| i + 1)),
        };
        return (at_rank(at), high);
    }
    let at = before.and_then(|b| index_of(sorted, b));
    (at_rank(at.and_then(|i| i.checked_sub(1))), at_rank(at))
}

fn place(
    board: &Board,
    coll: &str,
    neighbours: (Option<&str>, Option<&str>),
    except: Option<&str>,
) -> (f64, f64) {
    let mut sorted: Vec<&Value> = section(board, coll)
        .values()
        .filter(|d| except.is_none_or(|e| d["id"].as_str() != Some(e)))
        .collect();
    sorted.sort_by(|a, b| by_rank(a, b));
    match bounds(board, coll, &sorted, neighbours) {
        (None, Some(high)) => (high - RANK_STEP, f64::INFINITY),
        (Some(low), None) => (low + RANK_STEP, f64::INFINITY),
        (Some(low), Some(high)) => (f64::midpoint(low, high), high - low),
        (None, None) => (RANK_STEP, f64::INFINITY),
    }
}

fn check_neighbour(
    board: &Board,
    coll: &str,
    id: Option<&str>,
    except: Option<&str>,
) -> Result<(), BoardError> {
    let Some(id) = id else { return Ok(()) };
    if !section(board, coll).contains_key(id) {
        return Err(refuse(format!("no {coll} doc \"{id}\" to order against.")));
    }
    if Some(id) == except {
        return Err(refuse(format!("{id} cannot be placed next to itself.")));
    }
    Ok(())
}

/// The rank that puts a doc just before `before`, just after `after`, or between them when both
/// are given. `except` is the doc being moved, left out of the neighbour search. No ids: the end.
///
/// # Errors
/// Refused when a neighbour does not exist or is the moved doc itself.
pub fn rank_between(
    board: &mut Board,
    coll: &str,
    before: Option<&str>,
    after: Option<&str>,
    except: Option<&str>,
) -> Result<f64, BoardError> {
    check_neighbour(board, coll, before, except)?;
    check_neighbour(board, coll, after, except)?;
    if before.is_none() && after.is_none() {
        return Ok(rank_end(board, coll));
    }
    let (rank, gap) = place(board, coll, (before, after), except);
    if gap >= RANK_GAP {
        return Ok(rank);
    }
    renumber(board, coll);
    Ok(place(board, coll, (before, after), except).0)
}
