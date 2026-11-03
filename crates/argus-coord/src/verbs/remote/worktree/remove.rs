//! `coord worktree remove | prune`: Argus removes a worktree behind its guards (merged and
//! clean needs nothing, discarding work needs the owner's entry naming it; a backup is written
//! first), or every leftover folder older than an age, or prunes the gone ones.

use argus_board::json::to_number;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::opts::text_opt;
use super::path::target_of;
use super::text::days_text;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::client::ClientError;
use crate::verbs::remote::support::link::{connect, list_of, project_of};
use crate::verbs::remote::support::scope::require_board;
use crate::verbs::remote::support::text::{amount_and_unit, cell, say_all, truthy};
use crate::who::caller;

/// The id the dashboard knows a worktree by.
pub(super) fn id_arg(value: Option<&String>, usage: &str) -> Result<String, CoordError> {
    match value.filter(|v| !v.is_empty()) {
        Some(typed) => Ok(target_of(typed)?.0),
        None => Err(CoordError::refuse(usage)),
    }
}

/// A call past its time limit is not a stopped dashboard (it answers, only late): the sentence says what may
/// still be running there and where to look. A refused connection stays "not running".
fn slow(e: ClientError, what: &str, check: &str) -> CoordError {
    match e {
        ClientError::Timeout(secs) => CoordError::refuse(format!(
            "the dashboard did not answer in time ({secs} s): {what} may still be running; check `{check}`"
        )),
        other => other.into(),
    }
}

const AGE_USAGE: &str = "--older-than takes an age such as 3d, 12h, 1w or a number of days.";

/// `3d`, `12h`, `1w` or a bare number of days, in days.
pub(super) fn age_arg(text: Option<&str>) -> Result<f64, CoordError> {
    let (n, unit) =
        amount_and_unit(text.unwrap_or_default()).ok_or_else(|| CoordError::refuse(AGE_USAGE))?;
    let days = match unit.as_str() {
        "" | "d" => n,
        "h" => n / 24.0,
        "w" => n * 7.0,
        _ => return Err(CoordError::refuse(AGE_USAGE)),
    };
    if days < 0.25 {
        return Err(CoordError::refuse(
            "--older-than goes down to 6h at the least (a thread deletes nothing younger; the owner's page can).",
        ));
    }
    Ok(days)
}

/// The thread that asks: the caller of the verb on this project's board.
pub(super) fn asker(call: &Call<'_>) -> Result<String, CoordError> {
    let board = require_board(call, project_of(call))?;
    Ok(caller(&board, &call.inv.cwd, &call.inv.env, &call.options)?.thread)
}

pub(super) fn reason_of(call: &Call<'_>) -> Result<Value, CoordError> {
    Ok(text_opt(&call.options, "reason")?.map_or(Value::Null, |r| json!(r)))
}

fn all_leftovers(call: &mut Call<'_>) -> Outcome {
    if !call.positional.is_empty() {
        return Err(CoordError::refuse(
            "--all-leftovers takes no folder: coord worktree remove --all-leftovers [--older-than 3d].",
        ));
    }
    let o = &call.options;
    if o.flag("branch") || o.flag("discard") || o.get("owner-entry").is_some() {
        return Err(CoordError::refuse(
            "--all-leftovers deletes leftover folders only: they have no branch and no work git knows, so --branch, --discard and --owner-entry do not apply (and no backup is possible).",
        ));
    }
    let older = match o.get("older-than") {
        None => None,
        Some(_) => Some(age_arg(text_opt(o, "older-than")?.as_deref())?),
    };
    let by = asker(call)?;
    let mut body = Map::new();
    if let Some(days) = older {
        body.insert("olderThanDays".to_owned(), js::number(days));
    }
    body.insert("reason".to_owned(), reason_of(call)?);
    body.insert("by".to_owned(), json!(by));
    let project = project_of(call).to_owned();
    let out = connect(call, false)?
        .post(
            &format!("/api/p/{project}/worktrees/leftovers/remove"),
            &Value::Object(body),
        )
        .map_err(|e| slow(e, "the deletion", "coord worktree leftovers"))?;
    say_all(call.inv, &leftover_lines(&out));
    Ok(())
}

fn leftover_lines(out: &Value) -> Vec<String> {
    let mut lines = vec![format!("{}.", cell(out.get("text")))];
    if truthy(out, "started") {
        let days = days_text(to_number(out.get("olderThanDays")));
        lines.push(format!("Argus checks and deletes them one after another in the background (limit: older than {days}; folders a removal left behind have none); coord worktree leftovers shows what is left."));
    }
    let mut counts: Vec<(String, usize)> = Vec::new();
    for skipped in list_of(out, "skipped") {
        let reason = cell(skipped.get("reason"));
        match counts.iter_mut().find(|(r, _)| *r == reason) {
            Some((_, n)) => *n += 1,
            None => counts.push((reason, 1)),
        }
    }
    lines.extend(
        counts
            .iter()
            .map(|(reason, n)| format!("skipped {n}: {reason}")),
    );
    lines
}

fn backup_lines(backup: &Value) -> Vec<String> {
    if !js::truthy(backup) {
        return Vec::new();
    }
    let files: Vec<String> = ["bundle", "patch", "untracked"]
        .iter()
        .filter(|f| truthy(backup, f))
        .map(|f| cell(backup.get(*f)))
        .collect();
    let mut lines = vec![format!("backup: {}", files.join(", "))];
    lines.extend(
        list_of(backup, "restore")
            .iter()
            .map(|r| format!("restore: {}", cell(Some(r)))),
    );
    lines
}

fn removed_lines(out: &Value) -> Vec<String> {
    let mut lines = vec![format!("{}.", cell(out.get("text")))];
    if truthy(out, "finishedByArgus") {
        lines.push("git could not delete the folder (it unregistered the worktree); Argus renamed it and deletes it in the background, links never followed. coord worktree leftovers shows it until it is gone.".to_owned());
    }
    if truthy(out, "branch") && !truthy(out, "deletedBranch") && out["branchNote"] == "not merged" {
        lines.push(format!(
            "worktree removed, branch {} kept (not merged): its commits are safe in it. To delete the branch too: coord worktree remove ... --branch --discard --owner-entry <the owner's entry id>",
            cell(out.get("branch"))
        ));
    }
    lines.extend(backup_lines(&out["backup"]));
    lines
}

/// `coord worktree remove <id|path> [--branch] [--discard --owner-entry <E-id>] [--reason "..."]`,
/// or `remove --all-leftovers [--older-than 3d]`.
pub(super) fn remove(call: &mut Call<'_>) -> Outcome {
    if call.options.flag("all-leftovers") {
        return all_leftovers(call);
    }
    if call.options.get("older-than").is_some() {
        return Err(CoordError::refuse(
            "--older-than goes with --all-leftovers.",
        ));
    }
    let id = id_arg(
        call.positional.first(),
        "name the worktree: coord worktree remove <id|path> [--branch] [--discard --owner-entry <id>] [--reason \"...\"].",
    )?;
    let discard = call.options.flag("discard");
    let owner_entry = text_opt(&call.options, "owner-entry")?;
    if owner_entry.is_some() && !discard {
        return Err(CoordError::refuse(
            "--owner-entry goes with --discard (the owner's say to discard the work in it).",
        ));
    }
    let by = asker(call)?;
    let body = json!({
        "id": id, "branch": call.options.flag("branch"), "discard": discard,
        "ownerEntry": owner_entry, "reason": reason_of(call)?, "by": by,
    });
    let project = project_of(call).to_owned();
    let out = connect(call, false)?
        .post(&format!("/api/p/{project}/worktrees/remove"), &body)
        .map_err(|e| slow(e, "the removal", "coord worktree list"))?;
    let lines = if truthy(&out, "leftover") {
        vec![
            format!("{}.", cell(out.get("text"))),
            format!(
                "job {}: Argus deletes it in the background; coord worktree leftovers shows it until it is gone.",
                cell(out.get("jobId"))
            ),
        ]
    } else {
        removed_lines(&out)
    };
    say_all(call.inv, &lines);
    Ok(())
}

/// Posts `{ id, reason, by }` to `/worktrees/<action>` and says the one line Argus answers with.
fn file_request(call: &mut Call<'_>, id: &Value, action: &str, what: &str, check: &str) -> Outcome {
    let by = asker(call)?;
    let body = json!({ "id": id, "reason": reason_of(call)?, "by": by });
    let project = project_of(call).to_owned();
    let out = connect(call, false)?
        .post(&format!("/api/p/{project}/worktrees/{action}"), &body)
        .map_err(|e| slow(e, what, check))?;
    call.inv.say(&[&format!("{}.", cell(out.get("text")))]);
    Ok(())
}

/// `coord worktree prune [<id|path>]`.
pub(super) fn prune(call: &mut Call<'_>) -> Outcome {
    let id = match call.positional.first().filter(|v| !v.is_empty()) {
        Some(typed) => json!(target_of(typed)?.0),
        None => Value::Null,
    };
    file_request(call, &id, "prune", "the prune", "coord worktree list")
}

/// `coord worktree release <id|path> [--reason "..."]`: the thread no longer needs it. Argus
/// checks that nothing uses it and nothing is lost, files the removal (the branch too when merged)
/// as a request and answers at once with its id (`R-n: Argus removes it now; the result comes to
/// your inbox`); anything else is refused with the reason.
pub(super) fn release(call: &mut Call<'_>) -> Outcome {
    let id = id_arg(
        call.positional.first(),
        "name the worktree: coord worktree release <id|path> [--reason \"...\"].",
    )?;
    file_request(
        call,
        &json!(id),
        "release",
        "the removal",
        "coord worktree requests",
    )
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_timeout_says_the_work_may_still_run_and_a_refused_connection_says_not_running() {
        let late = slow(
            ClientError::Timeout(120),
            "the removal",
            "coord worktree requests",
        );
        assert_eq!(
            late.to_string(),
            "the dashboard did not answer in time (120 s): the removal may still be running; check `coord worktree requests`"
        );
        let down = slow(
            ClientError::down(Some("ECONNREFUSED")),
            "the removal",
            "coord worktree requests",
        );
        assert_eq!(
            down.to_string(),
            "the dashboard is not running (dashboard start): ECONNREFUSED"
        );
    }
}
