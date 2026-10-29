//! `coord pr list [--status open|merged|closed] [--json]`.

use argus_board::json::section;
use serde_json::{Map, Value};

use crate::cli::exit::Outcome;
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::project_of;
use crate::verbs::remote::support::scope::require_board;
use crate::verbs::remote::support::text::cell;

const HEAD: [&str; 6] = ["pr", "status", "branch", "commit", "checks", "title"];

/// The fields of a record a reader (Brock's merge) is given.
const FIELDS: [&str; 15] = [
    "id",
    "title",
    "body",
    "head",
    "base",
    "headCommit",
    "status",
    "item",
    "checks",
    "mergeCommit",
    "mergedAt",
    "mergedBy",
    "createdAt",
    "closedAt",
    "replay",
];

fn shown(pr: &Value) -> Value {
    let fields: Map<String, Value> = FIELDS
        .iter()
        .map(|f| ((*f).to_owned(), pr.get(*f).cloned().unwrap_or(Value::Null)))
        .collect();
    Value::Object(fields)
}

fn short(sha: Option<&Value>) -> String {
    let sha = cell(sha);
    sha.get(..7).unwrap_or(&sha).to_owned()
}

fn row(pr: &Value) -> Vec<String> {
    let checks: Vec<&str> = pr["checks"]
        .as_array()
        .map(|c| c.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    vec![
        cell(pr.get("id")),
        cell(pr.get("status")),
        format!("{} -> {}", cell(pr.get("head")), cell(pr.get("base"))),
        short(pr.get("headCommit")),
        checks.join(" "),
        out::clip(&cell(pr.get("title")), 60),
    ]
}

/// `coord pr list`: every pull request of the project, oldest first. `--json` prints an array
/// of `{id, title, body, head, base, headCommit, status, item, checks, mergeCommit, mergedAt,
/// mergedBy, createdAt, closedAt, replay}`.
pub(super) fn list(call: &mut Call<'_>) -> Outcome {
    let key = project_of(call).to_owned();
    let board = require_board(call, &key)?;
    let status = call.options.text("status");
    let prs: Vec<Value> = section(&board, "prs")
        .values()
        .filter(|p| status.is_none_or(|s| p["status"] == s))
        .map(shown)
        .collect();
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &Value::Array(prs));
    } else if prs.is_empty() {
        call.inv.say(&["no pull requests."]);
    } else {
        let mut rows = vec![HEAD.map(str::to_owned).to_vec()];
        rows.extend(prs.iter().map(row));
        call.inv.say(&[&out::table(&rows)]);
    }
    Ok(())
}
