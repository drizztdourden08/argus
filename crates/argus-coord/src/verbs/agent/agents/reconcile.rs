//! `coord agent reconcile [--all] [--json]`: what the dashboard's reconciler found at its last
//! run, from the running dashboard (`GET /api/agents/reconcile`): counts per project and one line
//! per record that needs a look. The dashboard must be running; nothing is started for it.

use argus_board::json::truthy;
use argus_core::js;
use serde_json::Value;

use crate::cli::Call;
use crate::cli::exit::Outcome;
use crate::cli::out::{ago, as_json, say, table};
use crate::client::dashboard_client;
use crate::verbs::agent::base::show::{cell, clip};

/// `5 working, 2 idle`, or `no live agents`.
fn counts(project: &Value) -> String {
    let parts: Vec<String> = project
        .as_object()
        .map(|m| {
            m.iter()
                .filter(|(_, n)| js::truthy(n))
                .map(|(k, n)| format!("{} {k}", cell(n)))
                .collect()
        })
        .unwrap_or_default();
    if parts.is_empty() {
        "no live agents".to_owned()
    } else {
        parts.join(", ")
    }
}

fn finding(f: &Value) -> Vec<String> {
    vec![
        cell(&f["project"]),
        cell(&f["id"]),
        cell(&f["name"]),
        cell(&f["thread"]),
        cell(&f["class"]),
        clip(&cell(&f["detail"]), 90),
    ]
}

fn report(out: &mut dyn std::io::Write, run: &Value, now_ms: i64) {
    if !truthy(run, "ran") {
        say(out, &["the reconciler has not run yet."]);
        return;
    }
    let skipped = if truthy(run, "skipped") { ", skipped: the machine had slept" } else { "" };
    let head = format!(
        "last run {} ago ({} ms){skipped}",
        ago(run["at"].as_str(), now_ms),
        cell(&run["ms"])
    );
    say(out, &[&head]);
    let projects = run["projects"].as_object();
    for (key, project) in projects.into_iter().flatten() {
        say(out, &[&format!("{key}: {}", counts(project))]);
    }
    if projects.is_none_or(serde_json::Map::is_empty) {
        say(out, &["no project has agent records."]);
    }
    let findings = run["findings"].as_array().filter(|f| !f.is_empty());
    if let Some(findings) = findings {
        let mut rows = vec![["project", "agent", "name", "thread", "class", "detail"]
            .map(str::to_owned)
            .to_vec()];
        rows.extend(findings.iter().map(finding));
        say(out, &[&table(&rows)]);
    }
}

/// `coord agent reconcile`.
///
/// # Errors
/// The dashboard is not running, or it refuses.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let (all, json) = (call.options.flag("all"), call.options.flag("json"));
    let project = call.project.clone().unwrap_or_default();
    let paths = call.inv.env.paths(&call.inv.cwd);
    let client = dashboard_client(&call.inv.env, &paths, false)?;
    let path = if all {
        "/api/agents/reconcile".to_owned()
    } else {
        format!("/api/agents/reconcile?project={}", js::encode_uri_component(&project))
    };
    let found = client.get(&path)?;
    let now_ms = call.inv.clock.now_ms();
    if json {
        as_json(&mut *call.inv.stdout, &found);
    } else {
        report(&mut *call.inv.stdout, &found, now_ms);
    }
    Ok(())
}
