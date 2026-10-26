//! `coord build list [--json]`: every bundle with its status, tests, pass, fail and to-run
//! counts, and the build the owner was handed.

use serde_json::{Value, json};

use super::rows::{Row, build_rows};
use crate::cli::Call;
use crate::cli::exit::Outcome;
use crate::cli::out::{ago, as_json, table};
use crate::verbs::agent::base::{read, show::clip};

const HEAD: [&str; 12] = [
    "build", "status", "title", "thread", "tests", "pass", "fail", "to run", "closed", "launcher",
    "cmd", "",
];

/// Where Play comes from: the linked launcher (marked when retired or missing), or a legacy
/// bundle's own command.
fn starts(row: &Row) -> String {
    let named = row.launcher.clone().unwrap_or_default();
    match row.launcher_state {
        state @ ("retired" | "missing") => format!("{named} ({state})"),
        _ if row.launcher.is_some() => named,
        _ if row.cmd.as_deref().is_some_and(|c| !c.is_empty()) => "(its own)".to_owned(),
        _ => String::new(),
    }
}

fn cells(row: &Row, now_ms: i64) -> Vec<String> {
    let closed = row
        .closed_at
        .as_deref()
        .filter(|c| !c.is_empty())
        .map_or_else(String::new, |c| format!("{} ago", ago(Some(c), now_ms)));
    vec![
        row.build.clone(),
        row.status.clone().unwrap_or_default(),
        clip(row.title.as_deref().unwrap_or_default(), 30),
        row.thread.clone().unwrap_or_default(),
        row.tests.to_string(),
        row.pass.to_string(),
        row.fail.to_string(),
        row.to_run.to_string(),
        closed,
        starts(row),
        clip(row.cmd.as_deref().unwrap_or_default(), 60),
        if row.handed { "handed" } else { "" }.to_owned(),
    ]
}

/// `coord build list`.
///
/// # Errors
/// A bundle or launcher record that cannot be read.
pub fn run(call: &mut Call<'_>) -> Outcome {
    read(call, |board, run| {
        let rows = build_rows(board)?;
        let handed = board["handedBuild"].as_str().filter(|h| !h.is_empty());
        if run.options.flag("json") {
            let one = handed.map(|h| json!({ "build": h, "at": board["handedAt"].clone() }));
            let all: Vec<Value> = rows.iter().map(Row::to_json).collect();
            as_json(run.out, &json!({ "handed": one, "builds": all }));
            return Ok(());
        }
        let head = match handed {
            Some(h) => format!(
                "handed to the owner: {h} (since {})",
                ago(board["handedAt"].as_str(), run.now_ms)
            ),
            None => "handed to the owner: none".to_owned(),
        };
        run.say(&head);
        if rows.is_empty() {
            run.say("no bundle and no test carries a build tag.");
            return Ok(());
        }
        let mut lines = vec![HEAD.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>()];
        lines.extend(rows.iter().map(|r| cells(r, run.now_ms)));
        run.say(&table(&lines));
        Ok(())
    })
}
