//! `coord test run end <run>`, `reopen <run>` and `drop <run>`: a running run ends with its
//! summary (passed when every case passed, else reported), a reported run is passed by hand, a
//! finished run comes back as reported, and a dropped run is abandoned. The build made for a run
//! retires with it when it is passed or abandoned.

use argus_board::json::display;
use serde_json::{Map, json};

use super::{run_item, set_run};
use crate::cli::Outcome;
use crate::verbs::work::crud::{apply_result, op};
use crate::verbs::work::run::{Run, bad};

/// `coord test run end`.
pub(super) fn end(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let run = run_item(board, r.arg(0))?;
        let id = display(&run["id"]);
        let line = match run["stage"].as_str() {
            Some("running") => {
                let end = op(("plan-run-end", "items", &id), Map::new(), r.options)?;
                apply_result(board, &who, &end)?.outcome
            }
            Some("reported") => {
                set_run(board, &who, (&id, json!({ "stage": "passed" })), r)?;
                format!("{id}: passed.")
            }
            other => {
                return Err(bad(format!(
                    "{id} is {}: only a running or reported run ends.",
                    other.unwrap_or("not started")
                )));
            }
        };
        r.say(&[&line]);
        Ok(())
    })
}

/// `coord test run reopen`.
pub(super) fn reopen(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let run = run_item(board, r.arg(0))?;
        let id = display(&run["id"]);
        if !matches!(run["stage"].as_str(), Some("passed" | "abandoned")) {
            r.say(&[&format!("{id}: is not closed.")]);
            return Ok(());
        }
        set_run(board, &who, (&id, json!({ "stage": "reported" })), r)?;
        r.say(&[&format!("{id}: reopened (reported); the build made for it is back.")]);
        Ok(())
    })
}

/// `coord test run drop`.
pub(super) fn drop_run(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let run = run_item(board, r.arg(0))?;
        let id = display(&run["id"]);
        set_run(board, &who, (&id, json!({ "stage": "abandoned" })), r)?;
        r.say(&[&format!("{id}: abandoned; the build made for it retires with it.")]);
        Ok(())
    })
}
