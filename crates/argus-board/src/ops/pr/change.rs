//! The ops that change a pull request: update, merge, close, and the check run list.

use argus_core::clock::now_iso;
use serde_json::{Map, Value, json};

use super::fields;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc_mut, set};
use crate::limits::optional_text;
use crate::ops::announce::audit;
use crate::ops::result::OpResult;
use crate::ops::who::Who;
use crate::store::stamp;

const BY_MAX: usize = 60;

/// Put the fields on the record, stamp it and leave one audit line.
fn write(
    board: &mut Board,
    (id, said): (&str, &str),
    changes: Map<String, Value>,
    who: &Who,
) -> Result<(), BoardError> {
    if let Some(pr) = doc_mut(board, "prs", id) {
        for (key, value) in changes {
            set(pr, &key, value);
        }
    }
    stamp(board, "prs", id, &who.actor_of())?;
    audit(
        board,
        &who.actor_of(),
        &format!("{} {said} {id}", who.label()),
    );
    Ok(())
}

fn given(set: &Value, key: &str) -> bool {
    !set[key].is_null()
}

fn update_fields(set: &Value) -> Result<Map<String, Value>, BoardError> {
    let mut changes = Map::new();
    if given(set, "headCommit") {
        let commit = fields::commit("headCommit", &set["headCommit"])?;
        changes.insert("headCommit".into(), json!(commit));
    }
    if given(set, "title") {
        changes.insert("title".into(), json!(fields::title(&set["title"])?));
    }
    if given(set, "body") {
        changes.insert("body".into(), json!(fields::body(&set["body"])?));
    }
    if changes.is_empty() {
        return Err(refuse(
            "nothing to update: give a headCommit, title or body.",
        ));
    }
    Ok(changes)
}

/// Apply a `pr-update` op: a new head commit, title or body of an open pull request.
pub(super) fn update(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let id = fields::open(board, op)?;
    let changes = update_fields(&op["set"])?;
    write(board, (&id, "updated"), changes, who)?;
    Ok(OpResult::applied(format!("{id} updated")))
}

fn merged_by(set: &Value, who: &Who) -> Result<String, BoardError> {
    let named = optional_text("mergedBy", &set["mergedBy"])?;
    Ok(named
        .map(|n| n.chars().take(BY_MAX).collect())
        .unwrap_or_else(|| who.actor_of()))
}

/// Apply a `pr-merge` op: the merge commit Brock's tooling made, and who merged.
pub(super) fn merge(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let id = fields::open(board, op)?;
    let Some(commit) = fields::commit("mergeCommit", &op["set"]["mergeCommit"])? else {
        return Err(refuse("a merge needs the merge commit (mergeCommit)."));
    };
    let by = merged_by(&op["set"], who)?;
    let changes = Map::from_iter([
        ("status".to_owned(), json!("merged")),
        ("mergedAt".to_owned(), json!(now_iso())),
        ("mergeCommit".to_owned(), json!(commit)),
        ("mergedBy".to_owned(), json!(by)),
    ]);
    write(board, (&id, "merged"), changes, who)?;
    Ok(OpResult::applied(format!("{id} merged")))
}

/// Apply a `pr-close` op: an open pull request ends without a merge.
pub(super) fn close(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let id = fields::open(board, op)?;
    let changes = Map::from_iter([
        ("status".to_owned(), json!("closed")),
        ("closedAt".to_owned(), json!(now_iso())),
    ]);
    write(board, (&id, "closed"), changes, who)?;
    Ok(OpResult::applied(format!("{id} closed")))
}

fn check_list(value: &Value) -> Result<Vec<String>, BoardError> {
    let Some(list) = value.as_array() else {
        return Err(refuse("checks must be a list of check run ids like C-3."));
    };
    list.iter().map(fields::check_id).collect()
}

/// Apply a `pr-checks` op: the check runs of the pull request, whatever its status. Nothing is
/// written when the list is the one the record already has.
pub(super) fn checks(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let id = fields::existing(board, op)?;
    let list = check_list(&op["set"]["checks"])?;
    if board["prs"][id.as_str()]["checks"] == json!(list) {
        return Ok(OpResult::applied(format!("{id} checks unchanged")));
    }
    if let Some(pr) = doc_mut(board, "prs", &id) {
        set(pr, "checks", json!(list));
    }
    stamp(board, "prs", &id, &who.actor_of())?;
    Ok(OpResult::applied(format!("{id} checks updated")))
}
