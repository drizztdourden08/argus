//! `coord build`: the old bundle verbs, kept one release as aliases of the verbs of test runs
//! (`coord test run ...`): each prints the new verb and then does it. A bundle is a run now, so
//! `add` and `set` have nothing to make or change and point to the verbs that do.

mod migrate;
mod support;

use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::opt::{take_sub, unknown_sub};
use crate::verbs::work::test_run_alias;

const VERBS: &str = "add, set, rm, close, reopen, ready, draft, hand, unhand, list, migrate-links";

/// The refusal of a verb that has no run to make or change.
fn gone(old: &str, new: &str) -> Outcome {
    Err(CoordError::refuse(format!(
        "coord build {old} is gone: a bundle is a test run now. Use {new}."
    )))
}

/// Runs the sub-verb the first word names; the other words stay for it.
///
/// # Errors
/// A sub-verb that does not exist, or the refusal of the sub-verb.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let old = take_sub(&mut call.positional);
    let new = match old.as_str() {
        "add" => return gone("add", "coord test plan run <plan> --build <launcher>"),
        "set" => return gone("set", "coord item set <run> --field build=<launcher>"),
        "migrate-links" => return migrate::run(call),
        "rm" => "drop",
        "close" => "end",
        "reopen" | "ready" | "draft" | "hand" | "unhand" | "list" => old.as_str(),
        _ => return Err(unknown_sub("build", VERBS)),
    };
    test_run_alias(call, (&old, new))
}
