//! The `switch` doc in the shape the web reads. The writers give the phase of the process (the
//! Desktop phase while the Desktop steps run) and the Desktop steps; this module turns them into
//! the doc: `phase` (idle, countdown, handover, sync, switching, verifying, done, failed,
//! cancelled), `desktopPhase` and `desktopSteps` as the writer gave them, `steps` (the step log),
//! `threads` (the leads of the handover), `sync`, `countdownEndsAt` as ISO, `error` and
//! `blocker` (why it is not moving).

use argus_accounts::sync::Summary;
use argus_core::clock::iso_from_ms;
use serde_json::{Value, json};

use super::line::Line;

/// The phase of the doc for a phase of the process.
pub(crate) fn phase_of(raw: &str) -> &str {
    match raw {
        "closing" | "saving_login" | "restoring_login" | "scheduling_resume" | "starting"
        | "waiting_for_quit" => "switching",
        "syncing" => "sync",
        "confirming" | "verifying" => "verifying",
        other => other,
    }
}

/// One entry of `steps`.
pub(super) fn entry(at: i64, line: &Line) -> Value {
    let mut out = json!({
        "at": iso_from_ms(at),
        "stage": line.stage,
        "state": line.state,
        "text": line.text,
    });
    if let Some(detail) = &line.detail {
        out["detail"] = json!(detail);
    }
    out
}

/// What the sync wrote, for the `sync` field: the backup folder, the counts, and the error.
pub(crate) fn sync_info(summary: Option<&Summary>, error: Option<&str>) -> Value {
    let mut out = json!({});
    if let Some(s) = summary {
        let k = &s.drift.by_kind;
        out["backupDir"] = json!(s.backup_dir);
        out["counts"] = json!({
            "copied": s.copied,
            "updated": k.chat.saturating_sub(s.copied),
            "folders": k.folder + k.membership,
            "archived": k.archive,
        });
    }
    if let Some(e) = error {
        out["error"] = json!(e);
    }
    out
}

/// The `sync` field: the state of the Desktop sync step, with what the sync said.
fn sync_of(desktop: &Value, info: &Value) -> Value {
    let step = desktop
        .as_array()
        .into_iter()
        .flatten()
        .find(|s| s["id"] == "sync");
    let state = match step.and_then(|s| s["status"].as_str()) {
        Some(s @ ("running" | "done" | "failed" | "skipped")) => s,
        _ => "pending",
    };
    let mut out = if info.is_object() {
        info.clone()
    } else {
        json!({})
    };
    out["state"] = json!(state);
    if state == "failed" && out["error"].is_null() {
        out["error"] = step.map_or(Value::Null, |s| s["detail"].clone());
    }
    out
}

/// Why the process is not moving, or null.
fn blocker_of(raw: &Value, threads: &[Value], now: i64) -> Value {
    let phase = raw["phase"].as_str().unwrap_or("idle");
    if phase == "waiting_for_quit" {
        return json!("Claude must be quit from its tray icon before the switch can go on.");
    }
    if phase == "handover" {
        let waiting: Vec<&str> = threads
            .iter()
            .filter(|t| t["state"] == "found" || t["state"] == "asked")
            .filter_map(|t| t["title"].as_str())
            .collect();
        if !waiting.is_empty() {
            return json!(format!(
                "Waiting for the handover of {}.",
                waiting.join(", ")
            ));
        }
    }
    let overdue = raw["countdownEndsAt"]
        .as_i64()
        .is_some_and(|end| end <= now);
    if phase == "countdown" && overdue {
        return json!("The countdown ended and the switch has not started yet.");
    }
    Value::Null
}

/// The parts of the log the doc carries.
pub(super) struct Parts<'a> {
    pub entries: &'a [Value],
    pub threads: &'a [Value],
    pub sync: &'a Value,
}

/// The doc the web reads, from the doc a writer gave.
pub(super) fn shape(raw: &Value, parts: &Parts<'_>, now: i64) -> Value {
    let mut doc = raw.clone();
    let Some(object) = doc.as_object_mut() else {
        return doc;
    };
    let process = raw["phase"].as_str().unwrap_or("idle");
    let desktop = if raw["steps"].is_array() {
        raw["steps"].clone()
    } else {
        json!([])
    };
    let ends_at = match &raw["countdownEndsAt"] {
        Value::Number(n) => n.as_i64().map_or(Value::Null, |ms| json!(iso_from_ms(ms))),
        other => other.clone(),
    };
    object.remove("log");
    object.insert("phase".to_owned(), json!(phase_of(process)));
    object.insert("desktopPhase".to_owned(), json!(process));
    object.insert("sync".to_owned(), sync_of(&desktop, parts.sync));
    object.insert("desktopSteps".to_owned(), desktop);
    object.insert("steps".to_owned(), json!(parts.entries));
    object.insert("threads".to_owned(), json!(parts.threads));
    object.insert("countdownEndsAt".to_owned(), ends_at);
    object.insert("blocker".to_owned(), blocker_of(raw, parts.threads, now));
    object.entry("error").or_insert(Value::Null);
    doc
}
