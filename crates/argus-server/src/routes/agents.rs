//! The last run of the sub-agent reconciler: `GET /api/agents/reconcile[?project=k]` is
//! `{ran, at, ms, skipped, projects, findings}`, the counts per project and the findings behind
//! them, narrowed to one project by `project`. Before the first run it is `{ran: false, at:
//! null, projects: {}, findings: []}`.

use argus_agents::RunReport;
use argus_core::ApiError;
use serde_json::{Map, Value, json, to_value};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

/// The report with only one project's counts and findings.
fn narrowed(run: &RunReport, project: &str) -> RunReport {
    RunReport {
        projects: run
            .projects
            .iter()
            .filter(|(key, _)| key.as_str() == project)
            .map(|(key, counts)| (key.clone(), *counts))
            .collect(),
        findings: run
            .findings
            .iter()
            .filter(|f| f.project == project)
            .cloned()
            .collect(),
        ..run.clone()
    }
}

fn ran(run: &RunReport) -> Value {
    let mut body = Map::new();
    body.insert("ran".into(), json!(true));
    if let Value::Object(fields) = to_value(run).unwrap_or(Value::Null) {
        body.extend(fields);
    }
    Value::Object(body)
}

async fn reconcile(ctx: Ctx) -> Reply {
    let reconciler = ctx
        .svc
        .agent_reconciler
        .clone()
        .ok_or_else(|| ApiError::unavailable("the agent reconciler is not running"))?;
    let Some(run) = reconciler.last() else {
        return Ok(json_reply(
            200,
            &json!({ "ran": false, "at": null, "projects": {}, "findings": [] }),
        ));
    };
    let run = match ctx.query("project").filter(|p| !p.is_empty()) {
        Some(project) => narrowed(&run, project),
        None => run,
    };
    Ok(json_reply(200, &ran(&run)))
}

/// Registers the sub-agent routes.
pub fn register(r: &mut Router) {
    r.get("/api/agents/reconcile", reconcile);
}
