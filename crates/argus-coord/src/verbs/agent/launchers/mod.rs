//! `coord launcher`: the builds the owner can start from the board's Builds tab. The registered
//! agents keep them true: `master` for the main checkout, one per active build or worktree,
//! `--current` on the valid one, retired when merged or replaced.

mod keep;
mod link;
mod list;
mod name;
mod patch;
mod report;
mod retire;
mod row;
mod set;
mod show;
mod unlink;

use super::base::opt::{take_sub, unknown_sub};
use crate::cli::Call;
use crate::cli::exit::Outcome;

const VERBS: &str = "set, keep, retire, list, show, link, unlink";

/// Runs the sub-verb the first word names; the other words stay for it.
///
/// # Errors
/// A sub-verb that does not exist, or the refusal of the sub-verb.
pub fn run(call: &mut Call<'_>) -> Outcome {
    match take_sub(&mut call.positional).as_str() {
        "set" => set::run(call),
        "link" => link::run(call),
        "unlink" => unlink::run(call),
        "keep" => keep::run(call),
        "retire" => retire::run(call),
        "list" => list::run(call),
        "show" => show::run(call),
        _ => Err(unknown_sub("launcher", VERBS)),
    }
}
