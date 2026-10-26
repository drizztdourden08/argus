//! `coord proc`: the process monitor of the dashboard (Argus > Processes): the processes the
//! Overlord's rules take, their memory and CPU, the ones over a limit, and the kill
//! (`verbs/proc.mjs`). Rules and kills are the Overlord's; the dashboard checks it again.

mod kill;
mod list;
mod patch;
mod rules;

use super::support::link::connect;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::client::Client;
use crate::verbs::remote::support::link::take_sub;

const USAGE: &str = "coord proc list [--rule r] [--json] | rules [--json]
coord proc rule set <name> [--name-match re] [--cmd-match re] [--path-match re] [--ignore re]... [--min-mem MB] [--max-mem MB] [--max-group-mem MB] [--max-age 6h] [--idle 30m] [--orphan|--no-orphan] [--hung|--no-hung] [--killable|--not-killable] [--only-flagged|--all-listed] [--note \"...\"]   (none removes a field)
coord proc rule rm <name>
coord proc kill <pid> [--tree] --reason \"...\"      coord proc kill-rule <rule> [--older-than 6h] [--idle-only] [--reason \"...\"]";

/// The dashboard, never started by these verbs.
fn dashboard(call: &Call<'_>) -> Result<Client, CoordError> {
    Ok(connect(call, false)?)
}

/// `coord proc <list|rules|rule|kill|kill-rule|help>`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
    match sub.as_deref() {
        None | Some("list") => list::list(call),
        Some("rules") => rules::rules(call),
        Some("rule") => rules::rule(call),
        Some("kill") => kill::kill(call),
        Some("kill-rule") => kill::kill_rule(call),
        Some("help") => {
            call.inv.say(&[USAGE]);
            Ok(())
        }
        Some(other) => Err(CoordError::refuse(format!(
            "unknown proc command \"{other}\".\n{USAGE}"
        ))),
    }
}
