//! The edit of one item the first word names: the caller is checked, the verb plans the fields,
//! the shared `edit` op applies them, and the verb says what it did.

use argus_board::Board;
use argus_board::ops::OpResult;
use serde_json::{Map, Value};

use super::force::{attach, reason};
use super::images::{Pending, keep_all};
use crate::cli::{Options, Outcome};
use crate::verbs::work::crud::{apply_result, checked, op, rev_of};
use crate::verbs::work::run::{Res, Run};
use crate::who::Caller;

/// What a verb prints once the edit is done.
pub(super) enum Say {
    /// `<id>: <fields> updated (rev <n>).`
    Updated,
    /// This sentence after the id.
    Line(String),
}

/// The fields to set and what to say.
pub(super) struct Plan {
    pub set: Map<String, Value>,
    pub say: Say,
}

/// The names of the fields a set changes, without the `fields.` of an own field.
fn named(set: &Map<String, Value>, pictures: usize) -> String {
    let mut names: Vec<&str> = set
        .keys()
        .map(|k| k.strip_prefix("fields.").unwrap_or(k))
        .collect();
    if pictures > 0 {
        names.push("images");
    }
    names.join(", ")
}

fn said(plan: &Plan, id: &str, (rev, pictures): (&str, usize)) -> String {
    match &plan.say {
        Say::Updated => format!("{id}: {} updated (rev {rev}).", named(&plan.set, pictures)),
        Say::Line(line) => format!("{id}: {line}"),
    }
}

/// Prints the warnings of the rules that only warn, one line each.
pub(super) fn warnings(r: &mut Run<'_>, res: &OpResult) {
    let list = res.extra.get("warnings").and_then(Value::as_array);
    for text in list.into_iter().flatten().filter_map(Value::as_str) {
        r.say(&[&format!("warning: {text}")]);
    }
}

/// The `edit` op for the fields, `None` when there are none (a call that only adds pictures).
fn apply_set(
    board: &mut Board,
    who: &Caller,
    (id, set): (&str, &Map<String, Value>),
    (options, force): (&Options, Option<&str>),
) -> Res<Option<OpResult>> {
    if set.is_empty() {
        return Ok(None);
    }
    let rest = Map::from_iter([("set".to_owned(), Value::Object(set.clone()))]);
    let operation = attach(op(("edit", "items", id), rest, options)?, force);
    apply_result(board, who, &operation).map(Some)
}

/// Edits the item the first word names. `plan` runs on the board after the caller and the item
/// are checked, so a verb that needs the item (a blocker list) or another record (a to-do) can
/// make them in the same write.
pub(super) fn edit_item(
    r: &mut Run<'_>,
    pictures: &[Pending],
    plan: impl FnOnce(&mut Board, &Caller, &Value) -> Res<Plan>,
) -> Outcome {
    let force = reason(r.options)?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let id = r.arg(0);
        let item = checked(board, &who, ("items", "set"), id)?;
        let id = id.unwrap_or_default().to_owned();
        let plan = plan(board, &who, &item)?;
        let res = apply_set(
            board,
            &who,
            (id.as_str(), &plan.set),
            (r.options, force.as_deref()),
        )?;
        keep_all(board, r, (id.as_str(), &who), pictures)?;
        let rev = rev_of(board, "items", &id);
        r.say(&[&said(&plan, &id, (rev.as_str(), pictures.len()))]);
        if let Some(res) = res {
            warnings(r, &res);
        }
        Ok(())
    })
}

/// A plan that needs nothing from the board.
pub(super) fn fixed(
    set: Map<String, Value>,
    say: Say,
) -> impl FnOnce(&mut Board, &Caller, &Value) -> Res<Plan> {
    move |_, _, _| Ok(Plan { set, say })
}
