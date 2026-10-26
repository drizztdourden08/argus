//! `coord agent`: the sub-agents a thread runs, as board documents (`A-1`, `A-2`, ...). The thread
//! that starts one registers it, keeps its note current and closes it, so the board lists every
//! agent working right now. The coordinator acts on any thread's agents with `--for <thread>`.

mod bind;
mod fields;
mod find;
mod finish;
mod list;
mod note;
mod reconcile;
mod set;
mod show;
mod soft;
mod start;

use crate::cli::Call;
use crate::cli::exit::Outcome;
use crate::verbs::agent::base::opt::{take_sub, unknown_sub};
use finish::Ending;
use soft::Soft;

const VERBS: &str =
    "start, note, set, done, fail, stop, archive, unarchive, rm, restore, list, show, reconcile";

/// Runs the sub-verb the first word names; the other words stay for it.
///
/// # Errors
/// A sub-verb that does not exist, or the refusal of the sub-verb.
pub fn run(call: &mut Call<'_>) -> Outcome {
    match take_sub(&mut call.positional).as_str() {
        "start" => start::run(call),
        "note" => note::run(call),
        "set" => set::run(call),
        "done" => finish::run(call, Ending::Done),
        "fail" => finish::run(call, Ending::Failed),
        "stop" => finish::run(call, Ending::Stopped),
        "archive" => soft::run(call, Soft::Archive),
        "unarchive" => soft::run(call, Soft::Unarchive),
        "rm" => soft::run(call, Soft::Remove),
        "restore" => soft::run(call, Soft::Restore),
        "list" => list::run(call),
        "show" => show::run(call),
        "reconcile" => reconcile::run(call),
        _ => Err(unknown_sub("agent", VERBS)),
    }
}
