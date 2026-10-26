//! `coord run status | list`: the runs the dashboard knows, as a table or as JSON.

use argus_core::js;
use serde_json::{Value, json};

use super::Res;
use super::follow::seconds;
use super::names::need_rid;
use crate::cli::out;
use crate::cli::route::Call;
use crate::client::Client;
use crate::verbs::remote::support::link::{list_of, project_of};
use crate::verbs::remote::support::query::Query;
use crate::verbs::remote::support::text::{cell, say_all, truthy};

const HEAD: [&str; 9] = [
    "run",
    "name",
    "status",
    "exit",
    "origin",
    "by",
    "started",
    "took",
    "last line",
];

fn row(run: &Value, now: i64) -> Vec<String> {
    let status = cell(run.get("status"));
    let shown_status = if truthy(run, "orphan") {
        format!("{status} (orphan)")
    } else {
        status.clone()
    };
    let started = if truthy(run, "startedAt") {
        out::ago(run["startedAt"].as_str(), now)
    } else if status == "queued" {
        "queued".to_owned()
    } else {
        String::new()
    };
    let took = seconds(&run["startedAt"], &run["endedAt"], now);
    let last = out::clip(&cell(run.get("lastLine")), 50);
    vec![
        cell(run.get("id")),
        cell(run.get("name")),
        shown_status,
        cell(run.get("exitCode")),
        cell(run.get("origin")),
        cell(run.get("by")),
        started,
        took,
        last,
    ]
}

fn rows(runs: &[Value], now: i64) -> String {
    let mut table = vec![HEAD.map(str::to_owned).to_vec()];
    table.extend(runs.iter().map(|run| row(run, now)));
    out::table(&table)
}

fn detail(run: &Value, now: i64) -> Vec<String> {
    let mut lines = vec![
        rows(std::slice::from_ref(run), now),
        format!("command: {}", cell(run.get("cmd"))),
        format!("cwd: {}", cell(run.get("cwd"))),
    ];
    for (field, label) in [("error", "error"), ("reason", "reason"), ("agent", "agent")] {
        if js::truthy(&run[field]) {
            lines.push(format!("{label}: {}", cell(run.get(field))));
        }
    }
    lines
}

/// `coord run status [<R-n>] [--json]`.
pub(super) fn status(call: &mut Call<'_>, client: &Client) -> Res<()> {
    let project = project_of(call).to_owned();
    let now = call.inv.clock.now_ms();
    if let Some(typed) = call.positional.first().filter(|p| !p.is_empty()) {
        let rid = need_rid(Some(typed))?;
        let answer = client.get(&format!("/api/p/{project}/runs/{rid}"))?;
        if call.options.flag("json") {
            out::as_json(&mut call.inv.stdout, &json!({ "run": answer["run"] }));
        } else {
            say_all(call.inv, &detail(&answer["run"], now));
        }
        return Ok(());
    }
    let answer = client.get(&format!("/api/p/{project}/runs?limit=10"))?;
    print_runs(call, &answer, now, "no runs yet.");
    Ok(())
}

fn print_runs(call: &mut Call<'_>, answer: &Value, now: i64, none: &str) {
    let runs = list_of(answer, "runs");
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &json!({ "runs": runs }));
    } else if runs.is_empty() {
        call.inv.say(&[none]);
    } else {
        call.inv.say(&[&rows(runs, now)]);
    }
}

/// `coord run list [--status s] [--name n] [--limit n] [--json]`.
pub(super) fn list(call: &mut Call<'_>, client: &Client) -> Res<()> {
    let project = project_of(call).to_owned();
    let mut query = Query::new();
    for key in ["status", "name", "limit"] {
        if let Some(value) = call.options.text(key) {
            query = query.with(key, value);
        }
    }
    let answer = client.get(&format!("/api/p/{project}/runs{}", query.suffix()))?;
    let now = call.inv.clock.now_ms();
    print_runs(call, &answer, now, "no runs match.");
    Ok(())
}
