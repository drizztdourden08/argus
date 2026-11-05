//! MCP resources: read-only documents a client can attach to a conversation without a tool
//! call. Same data as the tools; the uris are the ones DESIGN 7.1 lists.

use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Value, json};

use crate::backend::{LogQuery, McpBackend, RunFilter};
use crate::error::ToolError;
use crate::json::text;
use crate::project::require_project;
use crate::read_tools::portfolio_cards;
use crate::runs_view::RUNS_DOWN;
use crate::summary::board_summary;
use crate::to_run::tests_to_run;

static URI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-z]+)://([^/]+)(/.*)?$").expect("a fixed pattern"));
static RUN_LOG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^/(R-\d+)/log$").expect("a fixed pattern"));

fn document(uri: &str, value: &Value) -> Value {
    let body = serde_json::to_string_pretty(value).unwrap_or_default();
    json!({ "contents": [{ "uri": uri, "mimeType": "application/json", "text": body }] })
}

/// The resources: the overview, and three per project.
#[must_use]
pub fn list_resources(backend: &dyn McpBackend) -> Vec<Value> {
    let mut out = vec![json!({
        "uri": "portfolio://overview", "name": "portfolio-overview",
        "title": "Argus overview", "description": "One card per project.",
        "mimeType": "application/json"
    })];
    for key in backend.project_keys() {
        out.push(json!({
            "uri": format!("board://{key}/summary"), "name": format!("{key}-summary"),
            "title": format!("{key}: board summary"),
            "description": "Threads, items, questions, tests, locks, runs.",
            "mimeType": "application/json"
        }));
        out.push(json!({
            "uri": format!("board://{key}/tests/to-run"), "name": format!("{key}-tests-to-run"),
            "title": format!("{key}: manual tests to run"),
            "description": "Manual tests not run yet, grouped by test run.",
            "mimeType": "application/json"
        }));
        out.push(json!({
            "uri": format!("runs://{key}/active"), "name": format!("{key}-active-runs"),
            "title": format!("{key}: active runs"),
            "description": "Queued and running runs.", "mimeType": "application/json"
        }));
    }
    out
}

/// The four resource templates.
#[must_use]
pub fn list_resource_templates() -> Vec<Value> {
    vec![
        json!({ "uriTemplate": "board://{project}/summary", "name": "board-summary",
            "title": "Board summary", "mimeType": "application/json" }),
        json!({ "uriTemplate": "board://{project}/tests/to-run", "name": "tests-to-run",
            "title": "Manual tests to run", "mimeType": "application/json" }),
        json!({ "uriTemplate": "runs://{project}/active", "name": "active-runs",
            "title": "Active runs", "mimeType": "application/json" }),
        json!({ "uriTemplate": "runs://{project}/{run}/log", "name": "run-log",
            "title": "Run log (last 500 lines)", "mimeType": "text/plain" }),
    ]
}

fn unknown(uri: &str) -> ToolError {
    ToolError::not_found(format!("unknown resource {uri}"))
}

fn active_runs(backend: &dyn McpBackend, key: &str) -> Value {
    let filter = RunFilter {
        limit: Some(500),
        ..RunFilter::default()
    };
    let runs: Vec<Value> = backend
        .run_list(Some(key), &filter)
        .into_iter()
        .filter(|r| matches!(text(r, "status"), Some("queued" | "running")))
        .collect();
    json!({ "runs": runs })
}

fn run_log(backend: &dyn McpBackend, uri: &str, key: &str, id: &str) -> Result<Value, ToolError> {
    if !backend.runs_up() {
        return Err(ToolError::unavailable(RUNS_DOWN));
    }
    if backend.run_get(key, id).is_none() {
        return Err(ToolError::not_found(format!("no run {id} in {key}")));
    }
    let query = LogQuery {
        tail: Some(500),
        ..LogQuery::default()
    };
    let out = backend.run_logs(key, id, &query);
    let lines: Vec<String> = out["lines"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|l| {
            let line = text(l, "text").unwrap_or_default();
            let plain = line
                .strip_suffix("\r\n")
                .or_else(|| line.strip_suffix('\n'));
            plain.unwrap_or(line).to_owned()
        })
        .collect();
    Ok(json!({ "contents": [{ "uri": uri, "mimeType": "text/plain", "text": lines.join("\n") }] }))
}

/// Read one resource: `{contents: [...]}`.
pub fn read_resource(backend: &dyn McpBackend, uri: &str) -> Result<Value, ToolError> {
    let caps = URI.captures(uri).ok_or_else(|| unknown(uri))?;
    let (scheme, host) = (&caps[1], &caps[2]);
    let rest = caps.get(3).map_or("", |m| m.as_str());
    match (scheme, rest) {
        ("portfolio", "") if host == "overview" => {
            Ok(document(uri, &json!({ "cards": portfolio_cards(backend) })))
        }
        ("board", "/summary") => {
            let key = require_project(backend, Some(host))?;
            Ok(document(uri, &board_summary(backend, &key)))
        }
        ("board", "/tests/to-run") => {
            let key = require_project(backend, Some(host))?;
            Ok(document(uri, &tests_to_run(backend, &key)))
        }
        ("runs", "/active") => {
            let key = require_project(backend, Some(host))?;
            Ok(document(uri, &active_runs(backend, &key)))
        }
        ("runs", _) => {
            let log = RUN_LOG.captures(rest).ok_or_else(|| unknown(uri))?;
            let key = require_project(backend, Some(host))?;
            run_log(backend, uri, &key, &log[1])
        }
        _ => Err(unknown(uri)),
    }
}
