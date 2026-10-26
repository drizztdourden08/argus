//! `coord proc list`: the listing of the last sample, the biggest first.

use argus_board::json::to_number;
use argus_core::js;
use serde_json::Value;
use std::fmt::Write;

use super::dashboard;
use crate::cli::exit::Outcome;
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::list_of;
use crate::verbs::remote::support::query::uri_component;
use crate::verbs::remote::support::text::{cell, num, round, say_all, truthy};

/// `1536` as `1.5 GB`, `300` as `300 MB`.
pub(super) fn mem(mb: &Value) -> String {
    let n = to_number(Some(mb));
    if n >= 1024.0 {
        format!("{} GB", num(round(n / 102.4) / 10.0))
    } else {
        format!("{} MB", num(n))
    }
}

/// `45` as `45m`, `120` as `2h`, `4000` as `3d`.
pub(super) fn dur(minutes: &Value) -> String {
    let n = to_number(Some(minutes));
    if n >= 2880.0 {
        format!("{}d", num(round(n / 1440.0)))
    } else if n >= 60.0 {
        format!("{}h", num(round(n / 60.0)))
    } else {
        format!("{}m", num(n))
    }
}

fn status_of(p: &Value) -> String {
    let status = &p["status"];
    let mut words = vec![cell(Some(status))];
    for flag in p["flags"].as_array().into_iter().flatten() {
        if flag != status {
            words.push(cell(Some(flag)));
        }
    }
    words.join(",")
}

fn kill_of(p: &Value) -> String {
    if truthy(p, "protected") {
        format!("protected ({})", cell(p.get("protectedWhy")))
    } else if truthy(p, "killable") {
        "yes".to_owned()
    } else {
        "no".to_owned()
    }
}

fn row(p: &Value) -> Vec<String> {
    let cpu = match p.get("cpuPct") {
        None | Some(Value::Null) => "-".to_owned(),
        other => cell(other),
    };
    let idle = if truthy(p, "idleMin") {
        dur(&p["idleMin"])
    } else {
        "-".to_owned()
    };
    let what = match (p.get("hint"), p.get("project")) {
        (Some(hint), _) if !hint.is_null() => cell(Some(hint)),
        (_, project) => cell(project),
    };
    vec![
        cell(p.get("pid")),
        cell(p.get("name")),
        cell(p.get("rule")),
        mem(&p["memMB"]),
        cpu,
        dur(&p["ageMin"]),
        idle,
        status_of(p),
        kill_of(p),
        what,
    ]
}

fn head(meta: &Value, count: usize) -> String {
    let plural = if count == 1 { "" } else { "es" };
    let total = match meta.get("totalMemMB") {
        None | Some(Value::Null) => mem(&Value::from(0)),
        Some(total) => mem(total),
    };
    let sampled = match meta.get("sampledAt") {
        None | Some(Value::Null) => "never".to_owned(),
        at => cell(at),
    };
    let mut text = format!("{count} process{plural}, {total}, sampled {sampled}");
    if js::truthy(&meta["error"]) {
        write!(text, "; last sample failed: {}", cell(meta.get("error"))).ok();
    }
    if js::truthy(&meta["rulesError"]) {
        write!(text, "; rules: {}", cell(meta.get("rulesError"))).ok();
    }
    text
}

/// `coord proc list [--rule r] [--json]`.
pub(super) fn list(call: &mut Call<'_>) -> Outcome {
    let path = match call.options.text("rule") {
        Some(rule) => format!("/api/procs?rule={}", uri_component(rule)),
        None => "/api/procs".to_owned(),
    };
    let answer = dashboard(call)?.get(&path)?;
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &answer);
        return Ok(());
    }
    let meta = &answer["meta"];
    if !js::truthy(meta) {
        call.inv.say(&[
            "no sample yet (the monitor samples every 30 s; it may be off: GET /api/health).",
        ]);
        return Ok(());
    }
    let procs = list_of(&answer, "procs");
    let head = head(meta, procs.len());
    if procs.is_empty() {
        call.inv.say(&[&head]);
        return Ok(());
    }
    let mut rows = vec![
        [
            "pid", "name", "rule", "mem", "cpu%", "age", "idle", "status", "kill", "what",
        ]
        .map(str::to_owned)
        .to_vec(),
    ];
    rows.extend(procs.iter().map(row));
    say_all(call.inv, &[head, out::table(&rows)]);
    Ok(())
}
