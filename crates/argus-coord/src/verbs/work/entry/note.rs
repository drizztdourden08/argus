//! `coord note <I-n|T-n> "<text>"`: an informal remark on an item or a test, kept as a `note`
//! entry (`N-n`). It never blocks anything. Only the thread that owns the record (or the
//! coordinator) adds one.

use argus_board::Board;
use argus_board::ops::create_note;
use serde_json::{Map, Value};

use crate::cli::{Options, Outcome};
use crate::verbs::work::crud::{apply, op, who_of};
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

const USAGE: &str = "usage: coord note <I-n|T-n> \"<text>\".";

/// `coord note`.
pub(super) fn note(r: &mut Run<'_>) -> Outcome {
    let Some(record) = r.arg(0).map(str::to_owned) else {
        return Err(bad(USAGE).into());
    };
    let text = r.arg_json(1);
    r.write(|board, r| {
        let who = r.begin(board)?;
        let id = create_note(board, &who_of(&who), &record, &text)?;
        r.say(&[&format!("{id} noted on {record}.")]);
        Ok(())
    })
}

/// The new text of a note, checked like a new note on the same record.
pub(super) fn reword(
    board: &mut Board,
    who: &Caller,
    id: &str,
    (text, options): (&Value, &Options),
) -> Res<()> {
    let set = Map::from_iter([("text".to_owned(), text.clone())]);
    let rest = Map::from_iter([("set".to_owned(), Value::Object(set))]);
    apply(board, who, &op(("edit", "entries", id), rest, options)?)
}
