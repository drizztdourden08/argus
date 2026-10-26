//! `coord worktree leftovers`: the folders directly under a folder rule that git no longer lists,
//! only files left: their size, files and age.

use argus_board::json::to_number;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::list::{by_path, fetch};
use super::text::{about_text, days_text, size_text};
use crate::cli::exit::Outcome;
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::connect;
use crate::verbs::remote::support::text::{cell, num, say_all, truthy};

fn is_leftover(doc: &Value) -> bool {
    matches!(doc["status"].as_str(), Some("leftover" | "deleting"))
}

fn age_of(doc: &Value) -> f64 {
    doc["leftover"]["ageDays"].as_f64().unwrap_or(-1.0)
}

fn job_text(job: &Value) -> String {
    let progress = if truthy(job, "total") {
        format!(" {}/{}", cell(job.get("removed")), cell(job.get("total")))
    } else {
        String::new()
    };
    let error = if truthy(job, "error") {
        format!(": {}", cell(job.get("error")))
    } else {
        String::new()
    };
    format!(" ({}{progress}{error})", cell(job.get("state")))
}

/// The note column: why a folder cannot go yet, or that it is going.
fn note_of(doc: &Value, limit: f64) -> String {
    let l = &doc["leftover"];
    if doc["status"] == "deleting" {
        let job = if js::truthy(&doc["job"]) {
            job_text(&doc["job"])
        } else {
            String::new()
        };
        return format!("deleting{job}");
    }
    let pending = truthy(l, "pending");
    let made = truthy(l, "made");
    let nested = format!("nested git {}", cell(l.get("nestedGit")));
    let young = format!("younger than {}d", num(limit));
    let age = if l["ageDays"].is_null() {
        0.0
    } else {
        to_number(l.get("ageDays"))
    };
    [
        (pending, "not measured yet".to_owned()),
        (truthy(l, "hasNestedGit"), nested.trim().to_owned()),
        (truthy(l, "isLink"), "a link".to_owned()),
        (
            truthy(l, "nodeModules"),
            "node_modules not counted".to_owned(),
        ),
        (truthy(l, "partial") && !pending, "partial".to_owned()),
        (made, "left by a removal: no age limit".to_owned()),
        (!pending && !made && age < limit, young),
    ]
    .into_iter()
    .filter_map(|(on, text)| on.then_some(text))
    .collect::<Vec<_>>()
    .join(", ")
}

fn row(doc: &Value, limit: f64, now: i64) -> Vec<String> {
    let l = &doc["leftover"];
    let size = match l.get("sizeMB") {
        None | Some(Value::Null) => String::new(),
        other => size_text(other),
    };
    let newest = if truthy(l, "newestAt") {
        format!(
            "{} ({}d)",
            out::ago(l["newestAt"].as_str(), now),
            cell(l.get("ageDays"))
        )
    } else {
        String::new()
    };
    vec![
        cell(doc.get("id")),
        cell(doc.get("path")),
        size,
        cell(l.get("files")),
        newest,
        note_of(doc, limit),
    ]
}

fn meta_json(meta: &Value) -> Value {
    if !js::truthy(meta) {
        return Value::Null;
    }
    let mut shown = Map::new();
    for field in [
        "scannedAt",
        "leftovers",
        "leftoverMB",
        "leftoverOld",
        "deleting",
        "jobs",
    ] {
        if let Some(value) = meta.get(field) {
            shown.insert(field.to_owned(), value.clone());
        }
    }
    Value::Object(shown)
}

fn head(meta: &Value, rows: usize, limit: f64) -> String {
    let plural = if rows == 1 { "" } else { "s" };
    let old = &meta["leftoverOld"];
    let count = match old.get("count") {
        None | Some(Value::Null) => "0".to_owned(),
        other => cell(other),
    };
    format!(
        "{rows} leftover folder{plural} ({}), {count} older than {} ({}): coord worktree remove --all-leftovers deletes those in the background; no backup is possible (git does not know them)",
        about_text(meta.get("leftoverMB")),
        days_text(limit),
        about_text(old.get("sizeMB"))
    )
}

/// `coord worktree leftovers [--json]`.
pub(super) fn leftovers(call: &mut Call<'_>) -> Outcome {
    let client = connect(call, false)?;
    let (docs, meta) = fetch(call, &client)?;
    let mut rows: Vec<Value> = docs.into_iter().filter(is_leftover).collect();
    rows.sort_by(|a, b| age_of(b).total_cmp(&age_of(a)).then_with(|| by_path(a, b)));
    if call.options.flag("json") {
        out::as_json(
            &mut call.inv.stdout,
            &json!({ "meta": meta_json(&meta), "leftovers": rows }),
        );
        return Ok(());
    }
    if !js::truthy(&meta) {
        call.inv.say(&["the dashboard has not scanned this project yet (its worktrees service may be off: GET /api/health)."]);
        return Ok(());
    }
    let now = call.inv.clock.now_ms();
    if rows.is_empty() {
        let ago = out::ago(meta["scannedAt"].as_str(), now);
        call.inv.say(&[&format!(
            "no leftover folders under the rules (scanned {ago})."
        )]);
        return Ok(());
    }
    let limit = match meta["leftoverOld"].get("days") {
        None | Some(Value::Null) => 1.0,
        days => to_number(days),
    };
    let mut table = vec![
        ["id", "folder", "size", "files", "newest change", "note"]
            .map(str::to_owned)
            .to_vec(),
    ];
    table.extend(rows.iter().map(|d| row(d, limit, now)));
    say_all(
        call.inv,
        &[head(&meta, rows.len(), limit), out::table(&table)],
    );
    Ok(())
}
