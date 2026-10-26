//! The run tools over the run service: reads of the table, and the hidden starts, stops and
//! pictures of an MCP client. A run service that is off is `unavailable`.

use std::sync::Arc;

use argus_launchers::{ListFilter, LogOptions, Origin, RunService, StartOptions};
use argus_mcp::{Image, LogQuery, RunFilter, StartOpts, ToolError};
use serde_json::{Value, json, to_value};

use super::errors::{read_only, run};
use crate::Services;
use crate::adapters::runs::RunHost;

type Runs = Arc<RunService<RunHost>>;

fn service(svc: &Services) -> Result<Runs, ToolError> {
    svc.runs
        .clone()
        .ok_or_else(|| ToolError::unavailable("the runs service is not running"))
}

fn writable(svc: &Services) -> Result<Runs, ToolError> {
    if svc.shadow {
        return Err(read_only("shadow mode: Argus starts and changes no run"));
    }
    service(svc)
}

fn json_of(row: &impl serde::Serialize) -> Value {
    to_value(row).unwrap_or(Value::Null)
}

/// Whether the run service is running.
pub fn up(svc: &Services) -> bool {
    svc.runs.is_some()
}

/// One run.
pub fn get(svc: &Services, key: &str, id: &str) -> Option<Value> {
    svc.runs.as_ref()?.get(key, id).map(|r| json_of(&r))
}

/// Runs of a project, or of every project; none while the service is off.
pub fn list(svc: &Services, key: Option<&str>, filter: &RunFilter) -> Vec<Value> {
    let Some(runs) = svc.runs.as_ref() else {
        return Vec::new();
    };
    let wanted = ListFilter {
        statuses: filter.status.iter().cloned().collect(),
        name: filter.name.clone(),
        limit: filter.limit.and_then(|n| i64::try_from(n).ok()),
    };
    let rows = match key {
        Some(key) => runs.list(key, &wanted),
        None => runs.list_all(&wanted),
    };
    rows.iter().map(json_of).collect()
}

fn log_options(query: &LogQuery) -> LogOptions {
    let number = |n: Option<u64>| n.and_then(|n| i64::try_from(n).ok());
    LogOptions {
        stream: query.stream.clone(),
        tail: number(query.tail),
        after: number(query.after),
        limit: None,
    }
}

/// `{lines, next, dropped}` of a run; empty when it cannot be read.
pub fn logs(svc: &Services, key: &str, id: &str, query: &LogQuery) -> Value {
    let page = svc
        .runs
        .as_ref()
        .and_then(|r| r.logs(key, id, &log_options(query)).ok());
    match page {
        Some(p) => json!({ "lines": p.lines, "next": p.next, "dropped": p.dropped }),
        None => json!({ "lines": [], "next": 0, "dropped": 0 }),
    }
}

/// Starts a defined run, hidden, from an MCP client.
pub fn start(
    svc: &Services,
    key: &str,
    name: &str,
    opts: &StartOpts,
) -> Result<(Value, bool), ToolError> {
    let runs = writable(svc)?;
    let options = StartOptions {
        origin: Origin::Mcp,
        by: Some(opts.by.clone()),
        agent: opts.agent.clone(),
        reason: opts.reason.clone(),
        visible: Some(false),
    };
    let out = runs.start(key, name, &options).map_err(|e| run(&e))?;
    Ok((json_of(&out.run), out.already))
}

/// Stops a queued or running run.
pub async fn stop(svc: &Services, key: &str, id: &str, by: &str) -> Result<Value, ToolError> {
    let stopped = writable(svc)?.stop(key, id, by).await;
    stopped.map(|r| json_of(&r)).map_err(|e| run(&e))
}

/// Attaches a picture to a run: `{id, url}`.
pub fn attach(
    svc: &Services,
    (key, id): (&str, &str),
    image: &Image,
    caption: &str,
    by: &str,
) -> Result<Value, ToolError> {
    let runs = writable(svc)?;
    let kept = runs.attach(key, id, &image.bytes, image.mime, caption, by);
    let info = kept.map_err(|e| run(&e))?;
    Ok(json!({ "id": info.id, "url": info.url }))
}
