//! `coord agent list [--all] [--thread T] [--status s] [--archived] [--deleted] [--json]`: the
//! running agents of every thread by default; `--all` adds the finished ones, `--status s` picks
//! one status, `--archived` and `--deleted` look at the hidden ones.

use argus_board::Board;
use argus_board::json::{display, section, strict_eq, truthy};
use serde_json::Value;

use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out::{ago, as_json, table};
use crate::verbs::agent::base::opt::value;
use crate::verbs::agent::base::show::{cell, clip};
use crate::verbs::agent::base::{Run, read};

const STATUSES: [&str; 6] = [
    "working",
    "waiting",
    "done",
    "failed",
    "stopped",
    "stop-requested",
];
const LIVE: [&str; 3] = ["working", "waiting", "stop-requested"];
const HEAD: [&str; 12] = [
    "agent", "thread", "name", "status", "source", "kind", "model", "why", "item", "updated",
    "task", "note",
];

/// The list filter: live docs by default, only the archived with `--archived`, only the trash
/// with `--deleted`.
#[must_use]
pub fn visible(doc: &Value, run: &Run<'_>) -> bool {
    if run.options.flag("deleted") {
        return truthy(doc, "deletedAt");
    }
    if truthy(doc, "deletedAt") {
        return false;
    }
    if run.options.flag("archived") {
        truthy(doc, "archivedAt")
    } else {
        !truthy(doc, "archivedAt")
    }
}

/// The ranked order of the board: `(a.rank ?? 0) - (b.rank ?? 0)`.
pub fn by_rank(a: &Value, b: &Value) -> std::cmp::Ordering {
    let rank = |d: &Value| d["rank"].as_f64().unwrap_or(0.0);
    rank(a).total_cmp(&rank(b))
}

/// What was asked for: `(thread, status, widened)`.
fn filters(run: &Run<'_>) -> Result<(Option<String>, Option<String>, bool), CoordError> {
    let thread = run
        .options
        .text("thread")
        .or_else(|| run.options.text("for"))
        .map(str::to_owned);
    let status = value(run.options, "status")?.map(str::to_owned);
    if status.as_deref().is_some_and(|s| !STATUSES.contains(&s)) {
        return Err(CoordError::refuse(format!(
            "--status must be one of {}.",
            STATUSES.join(", ")
        )));
    }
    let widened = ["all", "archived", "deleted"]
        .iter()
        .any(|f| run.options.flag(f));
    Ok((thread, status, widened))
}

/// The model column: what was asked for, `(ran X, MISMATCH)` when the transcript shows another
/// model, `(ran X)` when it only adds one.
fn model_cell(a: &Value) -> String {
    let asked = cell(&a["model"]);
    if !truthy(a, "modelUsed") {
        return asked;
    }
    let used = display(&a["modelUsed"]);
    if truthy(a, "modelMismatch") {
        format!("{asked} (ran {used}, MISMATCH)")
    } else if !strict_eq(&a["modelUsed"], &a["model"]) {
        format!("{asked} (ran {used})")
    } else {
        asked
    }
}

fn row(a: &Value, now_ms: i64) -> Vec<String> {
    let source = if a["source"].is_null() {
        "manual".to_owned()
    } else {
        display(&a["source"])
    };
    vec![
        cell(&a["id"]),
        cell(&a["thread"]),
        cell(&a["name"]),
        cell(&a["status"]),
        source,
        cell(&a["kind"]),
        model_cell(a),
        clip(&cell(&a["modelWhy"]), 40),
        cell(&a["item"]),
        ago(a["updatedAt"].as_str(), now_ms),
        clip(&cell(&a["task"]), 50),
        clip(&cell(&a["note"]), 40),
    ]
}

fn chosen<'a>(
    board: &'a Board,
    run: &Run<'_>,
    status: Option<&str>,
    widened: bool,
    thread: Option<&str>,
) -> Vec<&'a Value> {
    let wanted = |a: &Value| match status {
        Some(s) => a["status"].as_str() == Some(s),
        None => widened || a["status"].as_str().is_some_and(|s| LIVE.contains(&s)),
    };
    let mut agents: Vec<&Value> = section(board, "agents")
        .values()
        .filter(|a| visible(a, run) && wanted(a))
        .filter(|a| thread.is_none_or(|t| a["thread"].as_str() == Some(t)))
        .collect();
    agents.sort_by(|a, b| by_rank(a, b));
    agents
}

/// `coord agent list`.
///
/// # Errors
/// A `--status` that is not one of the statuses.
pub fn run(call: &mut Call<'_>) -> Outcome {
    read(call, |board, run| {
        let (thread, status, widened) = filters(run)?;
        let agents = chosen(board, run, status.as_deref(), widened, thread.as_deref());
        if run.options.flag("json") {
            let all = Value::Array(agents.into_iter().cloned().collect());
            as_json(run.out, &all);
            return Ok(());
        }
        let text = if agents.is_empty() {
            if widened || status.is_some() || thread.is_some() {
                "no agents match.".to_owned()
            } else {
                "no agents running.".to_owned()
            }
        } else {
            let mut rows = vec![HEAD.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>()];
            rows.extend(agents.iter().map(|a| row(a, run.now_ms)));
            table(&rows)
        };
        run.say(&text);
        Ok(())
    })
}
