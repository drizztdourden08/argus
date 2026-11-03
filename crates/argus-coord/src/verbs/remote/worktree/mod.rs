//! `coord worktree`: tell Argus where the project's worktrees live (`verbs/worktree.mjs`). The
//! dashboard looks at those places with read-only git, lists them on the board's Worktrees tab,
//! and wakes the registered thread when one is stale. A removal is done by the dashboard behind
//! its guards. Any registered thread may read and write the rules; `remove`, `prune` and `request`
//! are the project lead's or the rule thread's.

mod idle;
mod keep;
mod leftovers;
mod link;
mod list;
mod opts;
mod path;
mod remove;
mod request;
mod requests;
mod rule_add;
mod rules;
mod text;

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::take_sub;

const VERBS: &str = "worktree verb must be one of rule add|rm, rules, list, leftovers, keep, done, remove, prune, release, request, requests, link, unlink.";

fn rule(call: &mut Call<'_>) -> Outcome {
    let action = take_sub(call);
    match action.as_deref() {
        Some("add") => rules::rule_add(call),
        Some("rm" | "remove") => rules::rule_rm(call),
        None | Some("list") => rules::rules(call),
        Some(_) => Err(CoordError::refuse(
            "worktree rule must be add <path> or rm <id>.",
        )),
    }
}

/// `coord worktree <rule|rules|list|leftovers|keep|done|link|unlink|remove|prune|release|request|requests>`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
    match sub.as_deref() {
        Some("rule") => rule(call),
        Some("rules") => rules::rules(call),
        Some("list") => list::list(call),
        Some("leftovers") => leftovers::leftovers(call),
        Some("keep") => keep::keep(call),
        Some("done") => keep::done(call),
        Some("link") => link::link(call),
        Some("unlink") => link::unlink(call),
        Some("remove") => remove::remove(call),
        Some("prune") => remove::prune(call),
        Some("release") => remove::release(call),
        Some("request") => request::request(call),
        Some("requests") => requests::requests(call),
        _ => Err(CoordError::refuse(VERBS)),
    }
}
