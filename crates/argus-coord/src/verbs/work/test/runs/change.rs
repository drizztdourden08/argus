//! The shared shape of a run verb: open the run as the caller, check it, set fields, say it.

use argus_board::Board;
use argus_board::json::display;
use serde_json::Value;

use super::{run_item, set_run};
use crate::cli::Outcome;
use crate::verbs::work::run::{Res, Run};
use crate::who::Caller;

/// The caller, the live run an argument names, and its id.
pub(super) fn open(board: &mut Board, r: &Run<'_>) -> Res<(Caller, Value, String)> {
    let who = r.begin(board)?;
    let run = run_item(board, r.arg(0))?;
    let id = display(&run["id"]);
    Ok((who, run, id))
}

/// Sets `fields` on the run after `check`, then says what `say` makes of the run and its id.
pub(super) fn change(
    r: &mut Run<'_>,
    fields: Value,
    check: impl FnOnce(&Board, &Value) -> Res<()>,
    say: impl FnOnce(&Value, &str) -> String,
) -> Outcome {
    r.write(|board, r| {
        let (who, run, id) = open(board, r)?;
        check(board, &run)?;
        set_run(board, &who, (&id, fields), r)?;
        r.say(&[&say(&run, &id)]);
        Ok(())
    })
}
