//! `argus status [--json]`: health, pid, port and runs; exit 1 when the server is down.

use argus_core::Paths;
use argus_core::clock::{now_ms, parse_ms};
use serde_json::{Map, Value, json};

use super::live::up;
use super::tray_line;

/// The JSON of a running server (`impl` is `node` for the Node dashboard): the health answer with the pid and the address in front.
fn running_json(base: &str, pid: Option<u32>, health: &Value, tray: &Value) -> String {
    let mut out = Map::new();
    out.insert("running".into(), json!(true));
    out.insert("pid".into(), json!(pid));
    out.insert("url".into(), json!(format!("{base}/")));
    if let Some(fields) = health.as_object() {
        out.extend(fields.clone());
    }
    out.entry("impl").or_insert_with(|| json!("node"));
    if !tray.is_null() {
        out.insert("tray".into(), tray.clone());
    }
    serde_json::to_string_pretty(&out).unwrap_or_default()
}

fn uptime_seconds(started_at: &Value) -> i64 {
    let started = started_at
        .as_str()
        .and_then(parse_ms)
        .unwrap_or_else(now_ms);
    ((now_ms() - started).max(0) + 500) / 1000
}

fn print_running(label: &str, base: &str, pid: &str, health: &Value, tray: &Value) {
    let up = uptime_seconds(&health["startedAt"]);
    let version = health["version"].as_str().unwrap_or_default();
    println!("{label} running at {base}/ (pid {pid}, up {up} s, version {version})");
    let runs = &health["runs"];
    println!(
        "  runs: {} running, {} queued; streams: {}; scopes: {}",
        runs["running"],
        runs["queued"],
        health["sse"]["clients"],
        health["scopes"].as_array().map_or(0, Vec::len)
    );
    let watcher = &health["watcher"];
    let state = if watcher["ok"] == true {
        "ok"
    } else {
        "not running"
    };
    let errors = watcher["errors"].as_u64().filter(|n| *n > 0);
    let errors = errors.map_or_else(String::new, |n| format!(", {n} errors"));
    println!("  watcher: {state}{errors}");
    if !tray.is_null() {
        println!("{}", tray_line::line(tray));
    }
}

/// Prints the state of the server and answers 0 for a running one, 1 for none.
///
/// # Errors
///
/// None; the signature matches the other verbs.
pub async fn status(paths: &Paths, json: bool) -> Result<u8, String> {
    let tray = tray_line::info(paths);
    let Some(running) = up(paths).await else {
        if json {
            println!("{}", json!({ "running": false }));
        } else {
            println!("dashboard is not running");
            if !tray.is_null() {
                println!("{}", tray_line::line(&tray));
            }
        }
        return Ok(1);
    };
    let base = running.at.base();
    if json {
        let pid = running.at.record.as_ref().map(|r| r.pid);
        println!("{}", running_json(&base, pid, &running.health, &tray));
    } else {
        let label = if running.is_argus() {
            "dashboard"
        } else {
            "the Node dashboard"
        };
        print_running(label, &base, &running.at.pid_text(), &running.health, &tray);
    }
    Ok(0)
}
