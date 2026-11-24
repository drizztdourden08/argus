//! The checker's report: what needs a nudge, a delivery or the lead's eye. Archived and
//! deleted records are out of it; the board file keeps them. Running agents are listed too; a
//! working one untouched for 45 minutes is marked stale (informational).

use std::collections::HashSet;

use argus_board::Board;
use argus_board::agents::is_live;
use argus_board::json::{section, truthy};
use argus_core::clock::parse_ms;
use serde_json::{Value, json};

use super::builds::by_build;
use crate::cli::out::{ago, clip};
use crate::verbs::thread::docs::{age_ms, all, cell, field, list, object, or_null, visible};

const IDLE_STATUSES: [&str; 4] = ["done", "dispatched", "paused", "archived"];
const STALE_AGENT_MS: i64 = 45 * 60_000;
const TEXT_MAX: usize = 90;

/// The time a report is made at, and what counts as stale.
#[derive(Clone, Copy)]
pub struct Clock {
    /// Now, in epoch milliseconds.
    pub now_ms: i64,
    /// The minutes of silence after which a thread is stale.
    pub stale_minutes: f64,
}

fn is_stale_agent(agent: &Value, now_ms: i64) -> bool {
    let at = ["updatedAt", "startedAt"]
        .iter()
        .find_map(|k| agent[*k].as_str().filter(|s| !s.is_empty()));
    agent["status"] == "working"
        && at
            .and_then(parse_ms)
            .is_some_and(|t| now_ms - t > STALE_AGENT_MS)
}

fn ids(docs: &[&Value]) -> Vec<Value> {
    docs.iter().filter_map(|d| field(d, "id")).collect()
}

fn owner_of(archived: &HashSet<String>, thread: Option<Value>) -> Option<Value> {
    match thread {
        Some(Value::String(id)) if archived.contains(&id) => {
            Some(json!(format!("{id} (archived thread)")))
        }
        other => other,
    }
}

fn ask_rows(docs: &[&Value], archived: &HashSet<String>, now_ms: i64) -> Vec<Value> {
    docs.iter()
        .map(|d| {
            object(vec![
                ("id", field(d, "id")),
                ("thread", owner_of(archived, field(d, "thread"))),
                ("age", Some(json!(ago(d["createdAt"].as_str(), now_ms)))),
                ("text", Some(json!(clip(&cell(d.get("text")), TEXT_MAX)))),
            ])
        })
        .collect()
}

fn pick<'a>(docs: &[&'a Value], keep: impl Fn(&Value) -> bool) -> Vec<&'a Value> {
    docs.iter().copied().filter(|d| keep(d)).collect()
}

fn undelivered(board: &Board, archived: &HashSet<String>) -> Vec<Value> {
    section(board, "inbox")
        .iter()
        .filter(|(thread, messages)| !list(messages).is_empty() && !archived.contains(*thread))
        .map(|(thread, messages)| {
            let latest = list(messages).last().and_then(|m| field(m, "text"));
            object(vec![
                ("thread", Some(json!(thread))),
                (
                    "session",
                    Some(or_null(field(
                        &board["threads"][thread.as_str()],
                        "session",
                    ))),
                ),
                ("count", Some(json!(list(messages).len()))),
                ("latest", latest),
            ])
        })
        .collect()
}

fn lock_queues(board: &Board, now_ms: i64) -> Vec<Value> {
    section(board, "locks")
        .values()
        .filter(|l| !list(&l["queue"]).is_empty())
        .map(|l| {
            object(vec![
                ("resource", field(l, "resource")),
                ("holder", field(l, "holder")),
                ("heldFor", Some(json!(ago(l["since"].as_str(), now_ms)))),
                ("queue", field(l, "queue")),
            ])
        })
        .collect()
}

fn agent_rows(board: &Board, now_ms: i64) -> Vec<Value> {
    all(board, "agents")
        .into_iter()
        .filter(|a| is_live(a))
        .map(|a| {
            object(vec![
                ("id", field(a, "id")),
                ("thread", field(a, "thread")),
                ("agent", field(a, "name")),
                ("status", field(a, "status")),
                ("kind", field(a, "kind")),
                ("model", field(a, "model")),
                ("task", field(a, "task")),
                ("note", field(a, "note")),
                ("updatedAt", field(a, "updatedAt")),
                ("stale", Some(json!(is_stale_agent(a, now_ms)))),
            ])
        })
        .collect()
}

fn thread_rows(
    threads: &[&Value],
    clock: Clock,
) -> (Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>) {
    let live: Vec<&Value> = pick(threads, |t| {
        !IDLE_STATUSES.iter().any(|s| t["status"] == *s)
    });
    let stale = pick(&live, |t| {
        age_ms(&t["lastSeen"], clock.now_ms) / 60_000.0 > clock.stale_minutes
    })
    .iter()
    .map(|t| {
        object(vec![
            ("thread", field(t, "id")),
            ("session", field(t, "session")),
            ("lastSeen", field(t, "lastSeen")),
            ("status", field(t, "status")),
        ])
    })
    .collect();
    let blocked = pick(&live, |t| {
        t["status"] == "blocked" || truthy(&t["flags"], "blocked")
    })
    .iter()
    .map(|t| object(vec![("thread", field(t, "id")), ("note", field(t, "note"))]))
    .collect();
    let needs = ids(&pick(&live, |t| truthy(&t["flags"], "needsHuman")));
    let dispatched = pick(threads, |t| t["status"] == "dispatched")
        .iter()
        .map(|t| {
            object(vec![
                ("thread", field(t, "id")),
                ("since", field(t, "registeredAt")),
                ("subject", field(t, "subject")),
            ])
        })
        .collect();
    (stale, blocked, needs, dispatched)
}

type Part = Vec<(&'static str, Option<Value>)>;

#[allow(clippy::unnecessary_wraps)]
fn rows(list: Vec<Value>) -> Option<Value> {
    Some(Value::Array(list))
}

fn thread_part(board: &Board, clock: Clock, archived: &HashSet<String>) -> Part {
    let (stale, blocked, needs_human, dispatched) = thread_rows(&all(board, "threads"), clock);
    vec![
        ("stale", rows(stale)),
        ("undelivered", rows(undelivered(board, archived))),
        ("dispatched", rows(dispatched)),
        ("blocked", rows(blocked)),
        ("needsHuman", rows(needs_human)),
    ]
}

fn entry_part(board: &Board, clock: Clock, archived: &HashSet<String>) -> Part {
    let entries = visible(board, "entries");
    let asks = |kind: &str, status: &str| {
        let chosen = pick(&entries, |e| e["kind"] == kind && e["status"] == status);
        rows(ask_rows(&chosen, archived, clock.now_ms))
    };
    let requests = pick(&entries, |e| {
        e["kind"] == "question" && e["needs"] == "decision" && e["status"] == "open"
    });
    let messages = pick(&entries, |e| {
        ["message", "notice"].iter().any(|k| e["kind"] == *k) && e["status"] == "open"
    });
    vec![
        ("openQuestions", asks("question", "open")),
        ("pendingDecisions", asks("decision", "pending")),
        ("decisionRequests", rows(ids(&requests))),
        ("openMessages", rows(ids(&messages))),
    ]
}

fn test_part(board: &Board, clock: Clock, archived: &HashSet<String>) -> Part {
    let tests = visible(board, "tests");
    let handed = or_null(field(board, "handedBuild"));
    let manual = pick(&tests, |t| {
        t["mode"] == "manual" && t["status"] == "pending"
    });
    let failing = pick(&tests, |t| {
        ["fail", "partial", "blocked"]
            .iter()
            .any(|s| t["status"] == *s)
    });
    let actions = pick(&visible(board, "actions"), |a| a["status"] == "pending");
    vec![
        ("handedBuild", Some(handed.clone())),
        (
            "tests",
            Some(json!({
                "failing": ids(&failing),
                "manualToRun": ids(&manual),
                "manualToRunByBuild": by_build(&manual, &handed),
            })),
        ),
        (
            "pendingActions",
            rows(ask_rows(&actions, archived, clock.now_ms)),
        ),
    ]
}

/// The report of a board.
#[must_use]
pub fn report(board: &Board, clock: Clock) -> Value {
    let threads = all(board, "threads");
    let archived: HashSet<String> = pick(&threads, |t| t["status"] == "archived")
        .iter()
        .map(|t| cell(t.get("id")))
        .collect();
    let mut parts: Part = vec![
        ("project", field(board, "project")),
        ("artifact", Some(Value::Null)),
        ("toPush", Some(json!(0))),
    ];
    parts.extend(thread_part(board, clock, &archived));
    parts.extend(entry_part(board, clock, &archived));
    parts.extend(test_part(board, clock, &archived));
    parts.push(("lockQueues", rows(lock_queues(board, clock.now_ms))));
    parts.push(("agents", rows(agent_rows(board, clock.now_ms))));
    object(parts)
}
