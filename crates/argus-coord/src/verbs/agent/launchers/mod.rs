//! `coord launcher`: the builds the owner can start from the board's Builds tab. The registered
//! agents keep them true: `master` for the main checkout, one per active build or worktree,
//! `--current` on the valid one, retired when merged or replaced.

mod keep;
mod list;
mod name;
mod patch;
mod retire;
mod row;
mod set;
mod show;

use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};

const VERBS: &str = "set, keep, retire, list, show";

/// Runs the sub-verb the first word names; the other words stay for it.
///
/// # Errors
/// A sub-verb that does not exist, or the refusal of the sub-verb.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let sub = if call.positional.is_empty() {
        String::new()
    } else {
        call.positional.remove(0)
    };
    match sub.as_str() {
        "set" => set::run(call),
        "keep" => keep::run(call),
        "retire" => retire::run(call),
        "list" => list::run(call),
        "show" => show::run(call),
        _ => Err(CoordError::refuse(format!(
            "launcher verb must be one of {VERBS}."
        ))),
    }
}
