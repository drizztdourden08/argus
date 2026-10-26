//! The report as text, for a human. The same values the JSON holds, one section per list.

use argus_board::json::{display, nullish, shown};
use serde_json::Value;

use crate::cli::out::{ago, clip};
use crate::verbs::thread::docs::{cell, js_number, list};
use crate::verbs::thread::frame::Frame;

fn of(v: &Value, key: &str) -> String {
    shown(v.get(key))
}

fn or(v: &Value, key: &str, fallback: &str) -> String {
    nullish(v, key).unwrap_or_else(|| fallback.to_owned())
}

fn when(v: &Value, key: &str, now_ms: i64) -> String {
    ago(v[key].as_str(), now_ms)
}

fn joined(ids: &Value, sep: &str) -> String {
    let text = list(ids).iter().map(display).collect::<Vec<_>>().join(sep);
    if text.is_empty() {
        "none".to_owned()
    } else {
        text
    }
}

fn section(frame: &mut Frame<'_>, label: &str, rows: &Value, fmt: impl Fn(&Value) -> String) {
    let mut lines = vec![format!(
        "{label}: {}",
        if list(rows).is_empty() { "none" } else { "" }
    )];
    lines.extend(list(rows).iter().map(|r| format!("  - {}", fmt(r))));
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    frame.say(&refs);
}

fn by_build_line(g: &Value) -> String {
    let build = match &g["build"] {
        Value::Null => "no build".to_owned(),
        other => display(other),
    };
    let handed = if g["handed"] == true { " (handed)" } else { "" };
    let tests: Vec<String> = list(&g["tests"]).iter().map(display).collect();
    format!("{build}{handed}: {}", tests.join(" "))
}

/// Prints the report.
pub fn print(frame: &mut Frame<'_>, r: &Value, stale_minutes: f64) {
    let now = frame.now_ms;
    frame.say(&[&format!(
        "# check {}  (dashboard http://127.0.0.1:47615, coord board open)",
        display(&r["project"])
    )]);
    let label = format!("stale > {}m", js_number(stale_minutes));
    section(frame, &label, &r["stale"], |t| {
        format!(
            "{} ({}, seen {}, session \"{}\")",
            of(t, "thread"),
            of(t, "status"),
            when(t, "lastSeen", now),
            or(t, "session", "?")
        )
    });
    section(frame, "undelivered inbox", &r["undelivered"], |u| {
        format!(
            "{} x{} (session \"{}\"): {}",
            of(u, "thread"),
            of(u, "count"),
            or(u, "session", "?"),
            of(u, "latest")
        )
    });
    section(frame, "dispatched, not registered", &r["dispatched"], |d| {
        format!(
            "{} ({}, {})",
            of(d, "thread"),
            or(d, "subject", ""),
            when(d, "since", now)
        )
    });
    section(frame, "blocked", &r["blocked"], |b| {
        format!("{}: {}", of(b, "thread"), or(b, "note", ""))
    });
    section(frame, "needs human", &r["needsHuman"], display);
    section(frame, "open questions", &r["openQuestions"], |q| {
        format!(
            "{} {} {}: {}",
            of(q, "id"),
            of(q, "thread"),
            of(q, "age"),
            of(q, "text")
        )
    });
    section(frame, "pending decisions", &r["pendingDecisions"], |d| {
        format!(
            "{} for {} {}: {}",
            of(d, "id"),
            of(d, "thread"),
            of(d, "age"),
            of(d, "text")
        )
    });
    print_rest(frame, r);
}

fn print_rest(frame: &mut Frame<'_>, r: &Value) {
    let now = frame.now_ms;
    frame.say(&[&format!(
        "decision requests: {}   open messages: {}",
        joined(&r["decisionRequests"], " "),
        joined(&r["openMessages"], " ")
    )]);
    frame.say(&[&format!(
        "tests failing: {}   manual tests to run: {}",
        joined(&r["tests"]["failing"], " "),
        joined(&r["tests"]["manualToRun"], " ")
    )]);
    let handed = match &r["handedBuild"] {
        Value::Null => "none".to_owned(),
        other => display(other),
    };
    let label = format!("manual tests to run, by build (handed to the owner: {handed})");
    section(
        frame,
        &label,
        &r["tests"]["manualToRunByBuild"],
        by_build_line,
    );
    section(frame, "manual actions pending", &r["pendingActions"], |a| {
        format!(
            "{} {} {}: {}",
            of(a, "id"),
            of(a, "thread"),
            of(a, "age"),
            of(a, "text")
        )
    });
    section(frame, "lock queues", &r["lockQueues"], |l| {
        let queue: Vec<String> = list(&l["queue"]).iter().map(display).collect();
        format!(
            "{} held by {} for {}, waiting: {}",
            of(l, "resource"),
            of(l, "holder"),
            of(l, "heldFor"),
            queue.join(" > ")
        )
    });
    section(frame, "agents", &r["agents"], |a| {
        let stale = if a["stale"] == true { ", STALE" } else { "" };
        let said = clip(
            &cell(
                a.get("note")
                    .filter(|n| !n.is_null())
                    .or_else(|| a.get("task")),
            ),
            70,
        );
        format!(
            "{} {}/{} ({}, updated {}{stale}): {said}",
            of(a, "id"),
            of(a, "thread"),
            of(a, "agent"),
            of(a, "status"),
            when(a, "updatedAt", now)
        )
    });
}
