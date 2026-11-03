//! What the local remote shows the pages (docs/local-remote.md, the API section):
//!
//! - `GET /api/remote` is `{mode, replay: {prs, releases, packages, checks}, ciRoots, syncs}`: the
//!   switch, the counts of records that wait for the owner's go (`replay.state` is `pending`), the
//!   integration checkouts of `coord remote ci-root` (project key to folder) and the last full
//!   remote sync of each project (`coord remote synced`; project key to `{at, by, note}`)
//! - `GET /api/releases` is `{projects: [{project, channels}]}` and `GET /api/p/<key>/releases`
//!   is `{channels}`: versions per channel, newest first, with the files and the version
//!   `latest.yml` points at
//! - `GET /api/p/<key>/checks?commit=&pr=&run=` is `{runs}` newest first, with ids `C-n`, and
//!   `GET /api/p/<key>/checks/<C-n>/log?step=<i>` is the log text of a step; each run has `run`,
//!   the `R-n` of the run engine that made it (`POST /api/p/<key>/checks`), or null; a run that
//!   changed tracked files is `failed` with `guardNote`
//! - `GET /api/registry` is `{builds, consumers, scopeRedirected, consumersNote}`: the local
//!   package builds and the checkouts that use them

use std::sync::Arc;

use argus_core::ApiError;
use argus_core::remote::{ModeSource, ci_roots, syncs};
use argus_records::checks::{CheckRun, run_json};
use argus_records::replay_of;
use axum::http::header;
use serde_json::{Map, Value, json};

use super::blocking::blocking;
use super::params::{key, known_keys, records};
use crate::adapters::check_links::CheckLinks;
use crate::reply::{bytes_reply, json_reply};
use crate::router::Reply;
use crate::{Ctx, Router, Services};

/// The most of a log sent: the end of it when it is longer.
const LOG_MAX: u64 = 4 * 1024 * 1024;

fn pending_prs(svc: &Services, keys: &[String]) -> usize {
    let boards = svc.boards();
    let waiting = |pr: &&Value| replay_of(pr).is_some_and(|r| r.is_pending());
    keys.iter()
        .filter_map(|k| boards.board(k))
        .map(|board| {
            let prs = board["prs"].as_object();
            prs.map_or(0, |m| m.values().filter(waiting).count())
        })
        .sum()
}

async fn remote(ctx: Ctx) -> Reply {
    let svc = Arc::clone(&ctx.svc);
    let keys: Vec<String> = known_keys(&ctx).into_iter().collect();
    let counts = blocking(move || {
        let records = svc.records.as_ref();
        json!({
            "prs": pending_prs(&svc, &keys),
            "releases": records.map_or(0, |r| r.releases.pending()),
            "packages": records.map_or(0, |r| r.registry.pending()),
            "checks": records.map_or(0, |r| r.checks.pending()),
        })
    })
    .await?;
    let mode = ctx.svc.remote.mode().as_str();
    let roots: Map<String, Value> = ci_roots(&ctx.svc.paths.coord_home)
        .into_iter()
        .map(|(key, dir)| (key, json!(dir)))
        .collect();
    let last: Map<String, Value> = syncs(&ctx.svc.paths.coord_home)
        .into_iter()
        .map(|(key, record)| (key, record.to_json()))
        .collect();
    let body = json!({ "mode": mode, "replay": counts, "ciRoots": roots, "syncs": last });
    Ok(json_reply(200, &body))
}

async fn releases(ctx: Ctx) -> Reply {
    let records = records(&ctx)?;
    let projects = blocking(move || records.releases.projects_json()).await?;
    Ok(json_reply(200, &json!({ "projects": projects })))
}

async fn project_releases(ctx: Ctx) -> Reply {
    let (records, key) = (records(&ctx)?, key(&ctx)?);
    let channels = blocking(move || records.releases.channels_json(&key)).await?;
    Ok(json_reply(200, &json!({ "channels": channels })))
}

/// A run as the API shows it, with `run`: the `R-n` of the run engine that made it, or null. A
/// run the guard failed (it changed tracked files) is `failed` with the note in `guardNote`.
fn row_of(run: &CheckRun, links: &CheckLinks) -> Value {
    let mut row = run_json(run);
    row["run"] = json!(links.run_of(&run.project, &run.run_id));
    if let Some(note) = links.flag_of(&run.project, &run.run_id) {
        row["status"] = json!("failed");
        row["guardNote"] = json!(note);
    }
    row
}

async fn checks(ctx: Ctx) -> Reply {
    let (records, key) = (records(&ctx)?, key(&ctx)?);
    let (commit, pr) = (
        ctx.query("commit").map(str::to_owned),
        ctx.query("pr").map(str::to_owned),
    );
    let (links, run) = (
        CheckLinks::new(Arc::clone(&ctx.svc.db)),
        ctx.query("run").map(str::to_owned),
    );
    let rows = blocking(move || {
        let runs = records
            .checks
            .of_project(&key, commit.as_deref(), pr.as_deref());
        runs.iter()
            .map(|r| row_of(r, &links))
            .filter(|row| run.as_deref().is_none_or(|rid| row["run"] == rid))
            .collect::<Vec<Value>>()
    })
    .await?;
    Ok(json_reply(200, &json!({ "runs": rows })))
}

fn tail(path: &std::path::Path) -> Option<(Vec<u8>, bool)> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    let cut = len > LOG_MAX;
    file.seek(SeekFrom::Start(len.saturating_sub(LOG_MAX)))
        .ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some((bytes, cut))
}

async fn check_log(ctx: Ctx) -> Reply {
    let (records, key) = (records(&ctx)?, key(&ctx)?);
    let id = ctx.param("id").to_owned();
    let step: usize = ctx
        .query("step")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ApiError::bad_request("step must be the number of a step"))?;
    let found = blocking(move || {
        let run = records.checks.find(&key, &id)?;
        tail(&records.checks.step_log(&run, step)?)
    })
    .await?;
    let (bytes, cut) = found.ok_or_else(|| ApiError::not_found("no such log"))?;
    let len = bytes.len().to_string();
    let mut headers = vec![
        (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
        (header::CONTENT_LENGTH, len.as_str()),
        (header::CACHE_CONTROL, "no-store"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    ];
    if cut {
        headers.push((header::HeaderName::from_static("x-log-truncated"), "start"));
    }
    Ok(bytes_reply(200, bytes, &headers))
}

async fn registry(ctx: Ctx) -> Reply {
    let records = records(&ctx)?;
    let reader = Arc::clone(&records);
    let builds = blocking(move || reader.registry.builds()).await?;
    let consumers = records.consumers.get().await;
    let redirected = !consumers.list.is_empty();
    Ok(json_reply(
        200,
        &json!({
            "builds": builds, "consumers": consumers.list,
            "scopeRedirected": redirected, "consumersNote": consumers.note,
        }),
    ))
}

/// Registers the routes of the local remote.
pub fn register(r: &mut Router) {
    r.get("/api/remote", remote);
    r.get("/api/releases", releases);
    r.get("/api/p/:key/releases", project_releases);
    r.get("/api/p/:key/checks", checks);
    r.get("/api/p/:key/checks/:id/log", check_log);
    r.get("/api/registry", registry);
}
