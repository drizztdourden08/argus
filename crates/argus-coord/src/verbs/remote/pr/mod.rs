//! `coord pr`: local pull requests, the board records `PR-n` of the local remote
//! (`docs/local-remote.md`). They work on the board file alone, so they work with the dashboard
//! stopped. A merge is a local git merge done by Brock (`brock pr merge --local PR-n`); this verb
//! records its merge commit.

mod apply;
mod close;
mod list;
mod merge;
mod open;
mod root;

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::take_sub;

const SUBS: &str = "open, list, merge, close";

/// The `PR-n` a verb was given.
fn need_pr(given: Option<&String>) -> Result<String, CoordError> {
    let id = given.map_or("", |g| g.trim());
    let number = id.strip_prefix("PR-");
    if number.is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())) {
        Ok(id.to_owned())
    } else {
        Err(CoordError::refuse(
            "give the pull request id, like PR-3 (coord pr list shows them).",
        ))
    }
}

/// `coord pr <open|list|merge|close>`.
///
/// # Errors
/// A refusal of the verb or of the board, or Brock's reason for a merge it did not do.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
    match sub.as_deref() {
        Some("open") => open::open(call),
        Some("list") => list::list(call),
        Some("merge") => merge::merge(call),
        Some("close") => close::close(call),
        _ => Err(CoordError::refuse(format!(
            "pr verb must be one of {SUBS}."
        ))),
    }
}
