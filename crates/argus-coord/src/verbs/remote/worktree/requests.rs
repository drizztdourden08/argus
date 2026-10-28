//! `coord worktree requests`: the removal requests of the project, newest first, with the
//! outcome sentence of every target.

use serde_json::{Value, json};

use crate::cli::exit::Outcome;
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::{connect, list_of, project_of};
use crate::verbs::remote::support::text::{cell, say_all, truthy};

const NONE: &str = "no removal requests yet: coord worktree request <id|path>... | --finished | --all-leftovers (Argus removes them for you and tells your inbox).";

fn status_note(status: &str) -> &str {
    if status == "queued" {
        "waiting"
    } else {
        status
    }
}

fn what(request: &Value) -> String {
    match request["mode"].as_str() {
        Some("finished") => "every finished worktree".to_owned(),
        Some("leftovers") => "every old leftover".to_owned(),
        _ => {
            let n = list_of(request, "targets").len();
            format!("{n} worktree{}", if n == 1 { "" } else { "s" })
        }
    }
}

fn handled(request: &Value) -> String {
    let targets = list_of(request, "targets").len();
    if targets == 0 {
        return String::new();
    }
    let results = list_of(request, "results");
    let ok = results.iter().filter(|r| r["ok"] == true).count();
    format!(
        "  ({}/{targets} handled: {ok} removed, {} refused)",
        results.len(),
        results.len() - ok
    )
}

fn skipped(request: &Value) -> String {
    let skipped = &request["skipped"];
    if !truthy(skipped, "count") {
        return String::new();
    }
    let reasons: Vec<String> = list_of(skipped, "reasons")
        .iter()
        .map(|r| cell(Some(r)))
        .collect();
    format!(
        "  skipped {}: {}",
        cell(skipped.get("count")),
        reasons.join("; ")
    )
}

fn head(request: &Value, now: i64) -> String {
    let status = cell(request.get("status"));
    let branch = if truthy(request, "branch") {
        " + branches"
    } else {
        ""
    };
    let note = if truthy(request, "note") {
        format!("  {}", cell(request.get("note")))
    } else {
        String::new()
    };
    format!(
        "{} {}  {}{branch}  by {} {}{}{note}{}",
        cell(request.get("id")),
        status_note(&status),
        what(request),
        cell(request.get("by")),
        out::ago(request["at"].as_str(), now),
        handled(request),
        skipped(request)
    )
}

fn lines_of(request: &Value, now: i64) -> Vec<String> {
    let mut lines = vec![head(request, now)];
    lines.extend(list_of(request, "results").iter().map(|x| {
        let word = if x["ok"] == true {
            "removed"
        } else {
            "refused"
        };
        format!("    {word}: {}", cell(x.get("text")))
    }));
    lines
}

/// `coord worktree requests [--json]`.
pub(super) fn requests(call: &mut Call<'_>) -> Outcome {
    let project = project_of(call).to_owned();
    let answer = connect(call, false)?.get(&format!("/api/p/{project}/worktrees/requests"))?;
    let list = list_of(&answer, "requests");
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &json!({ "requests": list }));
        return Ok(());
    }
    if list.is_empty() {
        call.inv.say(&[NONE]);
        return Ok(());
    }
    let now = call.inv.clock.now_ms();
    let lines: Vec<String> = list.iter().flat_map(|r| lines_of(r, now)).collect();
    say_all(call.inv, &lines);
    Ok(())
}
