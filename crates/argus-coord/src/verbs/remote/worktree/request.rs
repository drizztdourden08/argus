//! `coord worktree request`: ASK Argus to remove, for a lead whose own session refuses `remove`
//! (irreversible local destruction). The verb deletes nothing: it records a request; the server's
//! worker removes through the same guards and tells the inbox.

use serde_json::{Value, json};

use super::idle::idle_option;
use super::opts::text_opt;
use super::remove::{age_arg, asker, id_arg, reason_of};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::{connect, project_of};
use crate::verbs::remote::support::text::cell;
use argus_core::js;

const ONE_THING: &str = "ask for one thing: coord worktree request <id|path>... [--branch] [--reason \"...\"], or --finished [--idle 6h] [--branch], or --all-leftovers [--older-than 3d] [--reason \"...\"].";
const SWITCHES: [&str; 4] = ["branch", "finished", "all-leftovers", "discard"];

/// A switch takes no value: `request --finished X:/w/a` names a worktree again. The words the
/// switches swallowed, in switch order.
fn swallowed(call: &Call<'_>) -> Vec<String> {
    SWITCHES
        .iter()
        .filter_map(|name| call.options.text(name))
        .filter(|text| *text != "true")
        .map(str::to_owned)
        .collect()
}

fn on(call: &Call<'_>, name: &str) -> bool {
    call.options.flag(name) || call.options.text(name).is_some()
}

fn refuse(text: &str) -> CoordError {
    CoordError::refuse(text)
}

/// What the flags allow together, checked before the dashboard is asked. The owner's entry, when given.
fn check_shape(
    call: &Call<'_>,
    named: usize,
    idle: Option<f64>,
) -> Result<Option<String>, CoordError> {
    let (all, finished, discard) = (
        on(call, "all-leftovers"),
        on(call, "finished"),
        on(call, "discard"),
    );
    if idle.is_some() && !finished {
        return Err(refuse(
            "--idle goes with --finished (coord worktree request --finished --idle 6h).",
        ));
    }
    if [named > 0, all, finished].iter().filter(|x| **x).count() != 1 {
        return Err(refuse(ONE_THING));
    }
    let owner_entry = text_opt(&call.options, "owner-entry")?;
    if owner_entry.is_some() && !discard {
        return Err(refuse(
            "--owner-entry goes with --discard (the owner's say to discard the work in it).",
        ));
    }
    if (all || finished) && (discard || owner_entry.is_some()) {
        return Err(refuse(
            "--finished and --all-leftovers never discard work: --discard --owner-entry go with named worktrees.",
        ));
    }
    if all && on(call, "branch") {
        return Err(refuse(
            "leftover folders have no branch: --branch does not apply to --all-leftovers.",
        ));
    }
    if call.options.get("older-than").is_some() && finished {
        return Err(refuse(
            "--older-than goes with named leftovers or --all-leftovers, not with --finished.",
        ));
    }
    Ok(owner_entry)
}

/// `coord worktree request <id|path>... [--branch] [--reason "..."] [--discard --owner-entry <E-id>]`,
/// `request --finished [--idle 6h] [--branch]`, or `request --all-leftovers [--older-than 3d]`.
pub(super) fn request(call: &mut Call<'_>) -> Outcome {
    call.positional.extend(swallowed(call));
    let idle = idle_option(call)?;
    let owner_entry = check_shape(call, call.positional.len(), idle)?;
    let older = match call.options.get("older-than") {
        None => None,
        Some(_) => Some(age_arg(text_opt(&call.options, "older-than")?.as_deref())?),
    };
    let by = asker(call)?;
    let targets = call
        .positional
        .iter()
        .map(|p| id_arg(Some(p), ""))
        .collect::<Result<Vec<_>, _>>()?;
    let mut body = json!({
        "targets": targets, "finished": on(call, "finished"), "allLeftovers": on(call, "all-leftovers"),
        "branch": on(call, "branch"), "discard": on(call, "discard"), "ownerEntry": owner_entry,
        "reason": reason_of(call)?, "by": by,
    });
    if let Some(days) = older {
        body["olderThanDays"] = js::number(days);
    }
    if let Some(minutes) = idle {
        body["idleMinutes"] = js::number(minutes);
    }
    let project = project_of(call).to_owned();
    let out: Value =
        connect(call, false)?.post(&format!("/api/p/{project}/worktrees/requests"), &body)?;
    call.inv.say(&[&cell(out.get("text"))]);
    Ok(())
}
