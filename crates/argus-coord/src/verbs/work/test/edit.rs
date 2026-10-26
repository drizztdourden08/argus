//! The frame every test change shares: the caller, the test it may change, the change, and the
//! stamp.

use argus_board::json::{display, doc, map_mut};
use argus_board::store::stamp;
use serde_json::Value;

use crate::cli::Outcome;
use crate::verbs::work::crud::checked;
use crate::verbs::work::run::{Res, Run};

/// Runs `mutate` on the test named by the first word and its item, when the caller may change
/// it, then stamps the test.
pub(super) fn edit(
    r: &mut Run<'_>,
    mutate: impl FnOnce(&mut Run<'_>, &mut Value, &Value) -> Res<()>,
) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let mut test = checked(board, &who, ("tests", "set"), r.arg(0))?;
        let id = display(&test["id"]);
        let item = doc(board, "items", &display(&test["item"]))
            .cloned()
            .unwrap_or(Value::Null);
        mutate(r, &mut test, &item)?;
        map_mut(board, "tests").insert(id.clone(), test);
        stamp(board, "tests", &id, &who.actor)?;
        Ok(())
    })
}
