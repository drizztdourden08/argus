//! `coord purge`, `restore` and `trash` (`verbs/purge.mjs`): the lead's only way to
//! hard-delete, and the way back. `rm` on any record is a soft delete (the trash); a purge removes
//! records for good, but only after writing a restorable backup under
//! `<COORD_HOME>/<project>/purged/`. A purge is a dry run until `--confirm <project key>`.
//! Never run by a thread: the lead and the owner's page only.

mod apply;
mod confirm;
mod criteria;
mod dry;
mod restore;
mod trash;

use argus_board::ops::Role;
use serde_json::Value;

use super::frame::{Frame, with_frame};
use crate::cli::Val;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::who::Caller;

/// An option value as text: a bare flag is `true`.
fn text_of(val: &Val) -> String {
    match val {
        Val::Flag => "true".to_owned(),
        Val::Text(text) => text.clone(),
    }
}

/// Only the lead purges, restores and reads the trash.
fn need_lead(who: &Caller, what: &str) -> Result<(), CoordError> {
    if who.role == Role::Lead {
        return Ok(());
    }
    Err(CoordError::refuse(format!(
        "only the lead can {what}. Ask with coord inbox send lead \"...\" or coord message."
    )))
}

/// The sum of the numeric counts of a plan.
fn total(counts: &serde_json::Map<String, Value>) -> f64 {
    counts.values().filter_map(Value::as_f64).sum()
}

/// Runs the typed verb (`purge`, `restore` or `trash`) with the words after it.
///
/// # Errors
/// Refused as the verb it runs refuses.
pub fn run_verb(frame: &mut Frame<'_>, verb: &str, rest: &[String]) -> Outcome {
    match verb {
        "purge" if confirm::confirmed(frame)? => apply::run(frame, rest),
        "purge" => dry::run(frame, rest),
        "restore" => restore::run(frame),
        "trash" => trash::run(frame),
        _ => Err(CoordError::refuse(
            "purge verb must be one of purge, restore, trash.",
        )),
    }
}

/// The verbs `purge`, `restore` and `trash`.
///
/// # Errors
/// Refused as the verb it runs refuses.
pub fn run(call: &mut Call<'_>) -> Outcome {
    with_frame(call, |frame| {
        let words = frame.args.to_vec();
        let verb = words.first().map_or("", String::as_str);
        run_verb(frame, verb, words.get(1..).unwrap_or_default())
    })
}
