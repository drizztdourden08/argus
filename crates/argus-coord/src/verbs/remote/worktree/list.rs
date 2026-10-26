//! `coord worktree list`: what the dashboard found under the rules of the project, the ones that
//! need a decision first.

use std::cmp::Ordering;
use std::fmt::Write;

use argus_core::js;
use serde_json::{Value, json};

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;
use crate::client::Client;
use crate::verbs::remote::support::collate;
use crate::verbs::remote::support::link::{connect, list_of, project_of};
use crate::verbs::remote::support::text::{cell, say_all, truthy};

const NEED: [&str; 3] = ["stale", "merged", "dirty-stale"];
const NOT_SCANNED: &str = "the dashboard has not scanned this project yet (its worktrees service may be off: GET /api/health).";

fn needs(doc: &Value) -> bool {
    doc["status"].as_str().is_some_and(|s| NEED.contains(&s))
}

/// The rows of the worktrees service of the project, and its summary of the last scan.
pub(super) fn fetch(call: &Call<'_>, client: &Client) -> Result<(Vec<Value>, Value), CoordError> {
    let project = project_of(call);
    let rows = client.get(&format!("/api/p/{project}/db/worktrees"))?;
    let docs = list_of(&rows, "docs")
        .iter()
        .map(|d| d["data"].clone())
        .collect();
    let meta = client.get(&format!("/api/p/{project}/db/meta/worktrees"))?;
    Ok((docs, meta["data"].clone()))
}

/// The paths in the order `localeCompare` gives.
pub(super) fn by_path(a: &Value, b: &Value) -> Ordering {
    collate::compare(&cell(a.get("path")), &cell(b.get("path")))
}

fn rules_of(meta: &Value) -> &[Value] {
    list_of(meta, "rules")
}

fn invalid_rule(id: &str, why: &str) -> String {
    format!(
        "; rule {id} is invalid: {why} (ignored; a rule path is the parent folder, no wildcards: coord worktree rule rm {id}, then add the folder with --include)"
    )
}

fn head(meta: &Value, docs: &[Value], now: i64) -> String {
    let plural = if docs.len() == 1 { "" } else { "s" };
    let mut text = format!(
        "{} worktree{plural}, {} need a decision, scanned {}",
        docs.len(),
        docs.iter().filter(|d| needs(d)).count(),
        out::ago(meta["scannedAt"].as_str(), now)
    );
    if truthy(meta, "gone") {
        write!(
            text,
            "; {} gone (folder deleted, git still lists it: coord worktree prune)",
            cell(meta.get("gone"))
        )
        .ok();
    }
    if truthy(meta, "tooMany") {
        write!(
            text,
            "; that is a lot (over {}): remove the ones you no longer need",
            cell(meta.get("manyAt"))
        )
        .ok();
    }
    let broken = |r: &Value| js::truthy(&r["error"]) || r["exists"] == false;
    if rules_of(meta).iter().any(broken) {
        text.push_str("; some rules have a problem (coord worktree rules --json)");
    }
    for rule in rules_of(meta).iter().filter(|r| js::truthy(&r["invalid"])) {
        let id = cell(rule.get("id"));
        let why = cell(rule.get("invalid"));
        text.push_str(&invalid_rule(&id, &why));
    }
    text
}

fn request_text(doc: &Value) -> String {
    let request = &doc["request"];
    if !js::truthy(request) {
        return String::new();
    }
    let state = if js::truthy(&request["answer"]) {
        " (answered)"
    } else {
        " (waiting)"
    };
    format!(
        "{} {}{state}",
        cell(request.get("id")),
        cell(request.get("kind"))
    )
}

fn row(doc: &Value, now: i64) -> Vec<String> {
    let touched = if truthy(doc, "lastTouchedAt") {
        out::ago(doc["lastTouchedAt"].as_str(), now)
    } else {
        String::new()
    };
    let count = |field: &str| match doc.get(field) {
        None | Some(Value::Null) => "0".to_owned(),
        other => cell(other),
    };
    let dirty = if truthy(doc, "dirty") {
        cell(doc.get("dirty"))
    } else {
        String::new()
    };
    vec![
        cell(doc.get("id")),
        cell(doc.get("status")),
        cell(doc.get("path")),
        cell(doc.get("branch")),
        touched,
        format!("{}/{}", count("ahead"), count("behind")),
        dirty,
        request_text(doc),
    ]
}

/// `coord worktree list [--json]`.
pub(super) fn list(call: &mut Call<'_>) -> Outcome {
    let client = connect(call, false)?;
    let (mut docs, meta) = fetch(call, &client)?;
    docs.sort_by(|a, b| needs(b).cmp(&needs(a)).then_with(|| by_path(a, b)));
    if call.options.flag("json") {
        out::as_json(
            &mut call.inv.stdout,
            &json!({ "meta": meta, "worktrees": docs }),
        );
        return Ok(());
    }
    if !js::truthy(&meta) {
        call.inv.say(&[NOT_SCANNED]);
        return Ok(());
    }
    let now = call.inv.clock.now_ms();
    let head = head(&meta, &docs, now);
    if docs.is_empty() {
        let none = if rules_of(&meta).is_empty() {
            "no rules yet: coord worktree rule add <path>."
        } else {
            "nothing found under the rules."
        };
        call.inv.say(&[&head, none]);
        return Ok(());
    }
    let mut rows = vec![
        [
            "id",
            "status",
            "path",
            "branch",
            "touched",
            "ahead/behind",
            "dirty",
            "request",
        ]
        .map(str::to_owned)
        .to_vec(),
    ];
    rows.extend(docs.iter().map(|d| row(d, now)));
    say_all(call.inv, &[head, out::table(&rows)]);
    Ok(())
}
