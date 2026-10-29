//! Starting a check run (`docs/local-remote.md`): `POST /api/p/:key/checks {commit?, pr?, check?,
//! job?, trigger?}` runs `brock ci` in the root of the project on the run engine (hidden, logs
//! kept, Stop and history are the run engine's) and answers `{run: "R-n"}`. When the run prints
//! its first line, the path of Brock's record, the pair is kept and `GET /api/p/:key/checks`
//! shows `run` on that record.

use std::sync::Arc;
use std::time::Duration;

use argus_core::clock::now_iso;
use argus_core::remote::ci_root_of;
use argus_core::{ApiError, ErrorCode};
use argus_launchers::{LogOptions, Origin, RunDef, RunKind, RunService, StartOptions, check_key};
use serde_json::{Value, json};

use super::params::key;
use super::runs::{refusal, service};
use super::write::{actor, object_body, writable};
use crate::adapters::check_links::{CheckLinks, run_id_of};
use crate::adapters::runs::RunHost;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{AuthKind, Ctx, Router, Services};

const NAME: &str = "check";
const PROGRAM: &str = "brock";
const TIMEOUT_SEC: u32 = 7200;
const PARALLEL: u32 = 8;
const POLL: Duration = Duration::from_millis(200);
const JOBS: [&str; 3] = ["gate", "upgrade", "all"];
const TRIGGERS: [&str; 4] = ["push", "pr", "manual", "release"];

/// What the caller asked `brock ci` to check. Every part was checked to be plain words, so the
/// command line needs no quoting.
#[derive(Debug, Default, PartialEq, Eq)]
struct Args {
    check: Option<String>,
    commit: Option<String>,
    pr: Option<String>,
    job: Option<String>,
    trigger: Option<String>,
}

fn text<'a>(body: &'a Value, name: &str) -> Result<Option<&'a str>, ApiError> {
    match &body[name] {
        Value::Null => Ok(None),
        Value::String(s) if s.trim().is_empty() => Ok(None),
        Value::String(s) => Ok(Some(s.trim())),
        _ => Err(ApiError::bad_request(format!("{name} must be a text"))),
    }
}

fn one_of(body: &Value, name: &str, words: &[&str]) -> Result<Option<String>, ApiError> {
    match text(body, name)? {
        Some(w) if words.contains(&w) => Ok(Some(w.to_owned())),
        Some(_) => Err(ApiError::bad_request(format!(
            "{name} must be one of {}",
            words.join(", ")
        ))),
        None => Ok(None),
    }
}

fn commit_of(body: &Value) -> Result<Option<String>, ApiError> {
    let Some(sha) = text(body, "commit")? else {
        return Ok(None);
    };
    let hex = sha.bytes().all(|b| b.is_ascii_hexdigit());
    if hex && (7..=40).contains(&sha.len()) {
        Ok(Some(sha.to_ascii_lowercase()))
    } else {
        Err(ApiError::bad_request(
            "commit must be 7 to 40 hex characters",
        ))
    }
}

fn pr_of(body: &Value) -> Result<Option<String>, ApiError> {
    let Some(id) = text(body, "pr")? else {
        return Ok(None);
    };
    let number = id.strip_prefix("PR-");
    if number
        .is_some_and(|n| !n.is_empty() && n.len() <= 9 && n.bytes().all(|b| b.is_ascii_digit()))
    {
        Ok(Some(id.to_owned()))
    } else {
        Err(ApiError::bad_request("pr must look like PR-3"))
    }
}

fn check_of(body: &Value) -> Result<Option<String>, ApiError> {
    let Some(name) = text(body, "check")? else {
        return Ok(None);
    };
    let plain = |b: u8| b.is_ascii_alphanumeric() || b"-_".contains(&b);
    if name.len() <= 40 && !name.starts_with('-') && name.bytes().all(plain) {
        Ok(Some(name.to_owned()))
    } else {
        Err(ApiError::bad_request(
            "check must be a plain word (letters, digits, - and _)",
        ))
    }
}

impl Args {
    fn parse(body: &Value) -> Result<Args, ApiError> {
        Ok(Args {
            check: check_of(body)?,
            commit: commit_of(body)?,
            pr: pr_of(body)?,
            job: one_of(body, "job", &JOBS)?,
            trigger: one_of(body, "trigger", &TRIGGERS)?,
        })
    }

    /// `<program> ci [check] [--commit c] [--pr p] [--job j] [--trigger t] --project key`.
    fn command(&self, program: &str, project: &str) -> String {
        let mut words = vec![program.to_owned(), "ci".to_owned()];
        words.extend(self.check.clone());
        for (flag, value) in [
            ("--commit", &self.commit),
            ("--pr", &self.pr),
            ("--job", &self.job),
            ("--trigger", &self.trigger),
        ] {
            if let Some(value) = value {
                words.extend([flag.to_owned(), value.clone()]);
            }
        }
        words.extend(["--project".to_owned(), project.to_owned()]);
        words.join(" ")
    }

    fn reason(&self) -> String {
        let of = self.pr.as_ref().or(self.commit.as_ref());
        format!("check {}", of.map_or("the working tree", String::as_str))
    }
}

/// The folder `brock ci` runs in: the integration checkout of `coord remote ci-root` when one
/// is set, else the root of the project from the boards and the config.
fn root_of(ctx: &Ctx, project: &str) -> Result<String, ApiError> {
    let known = ctx.svc.boards().projects();
    let root = ci_root_of(&ctx.svc.paths.coord_home, project)
        .or_else(|| known.into_iter().find(|p| p.key == project)?.root)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                format!(
                    "no root is known for {project} (coord portfolio add {project} --root <dir>)"
                ),
            )
        })?;
    if std::path::Path::new(&root).is_dir() {
        Ok(root)
    } else {
        Err(ApiError::bad_request(format!(
            "the root of {project} is not a folder: {root}"
        )))
    }
}

/// The program that runs a check: `brock`, or the `command` of the `checks` setting of
/// `config.json`, written as it is typed in a shell (Brock when it is not on the PATH, or a
/// test's stand-in).
fn program_of(ctx: &Ctx) -> String {
    let config = ctx.svc.config();
    let set = config
        .extra
        .get("checks")
        .and_then(|c| c["command"].as_str());
    let program = set.map(str::trim).filter(|c| !c.is_empty());
    program.unwrap_or(PROGRAM).to_owned()
}

fn definition(cmd: String, root: String) -> RunDef {
    RunDef {
        name: NAME.to_owned(),
        cmd,
        cwd: root,
        kind: RunKind::Test,
        timeout_sec: TIMEOUT_SEC,
        parallel: PARALLEL,
        defined_by: "argus".to_owned(),
        defined_at: now_iso(),
        ..RunDef::default()
    }
}

/// The first stdout line of a run, once it has one.
fn first_line(runs: &RunService<RunHost>, project: &str, rid: &str) -> Option<String> {
    let options = LogOptions {
        stream: Some("out".to_owned()),
        after: Some(0),
        limit: Some(1),
        ..LogOptions::default()
    };
    let page = runs.logs(project, rid, &options).ok()?;
    page.lines.into_iter().next().map(|l| l.text)
}

fn live(runs: &RunService<RunHost>, project: &str, rid: &str) -> bool {
    runs.get(project, rid).is_some_and(|r| r.status.is_live())
}

/// Keeps the pair a first line makes, when the line names a record of the project.
fn keep_pair(svc: &Services, (project, rid): (&str, &str), line: &str) {
    let Some(run_id) = run_id_of(project, line) else {
        return;
    };
    let links = CheckLinks::new(Arc::clone(&svc.db));
    if !links.link(project, &run_id, rid) {
        let detail = json!({ "run": rid, "record": run_id });
        svc.log.warn("check run not linked to its record", detail);
    }
}

/// Waits for the first line of the run, which names Brock's record, and keeps the pair. Ends with
/// the run, with the first line, or with the server.
async fn link_when_printed(
    svc: Arc<Services>,
    runs: Arc<RunService<RunHost>>,
    project: String,
    rid: String,
) {
    loop {
        if let Some(line) = first_line(&runs, &project, &rid) {
            keep_pair(&svc, (&project, &rid), &line);
            return;
        }
        if !live(&runs, &project, &rid) {
            return;
        }
        tokio::select! {
            () = tokio::time::sleep(POLL) => {}
            () = svc.stop.cancelled() => return,
        }
    }
}

fn options_of(ctx: &Ctx, body: &Value, args: &Args) -> StartOptions {
    let origin = if ctx.auth == Some(AuthKind::Page) {
        Origin::Page
    } else {
        Origin::Cli
    };
    StartOptions {
        origin,
        by: Some(actor(ctx, body)),
        agent: None,
        reason: Some(args.reason()),
        visible: Some(false),
    }
}

async fn start(ctx: Ctx) -> Reply {
    writable(&ctx)?;
    let runs = service(&ctx)?;
    let body = object_body(&ctx).await?;
    let project = key(&ctx)?;
    check_key(&project).map_err(|e| refusal(&e))?;
    let args = Args::parse(&body)?;
    let cmd = args.command(&program_of(&ctx), &project);
    let def = definition(cmd, root_of(&ctx, &project)?);
    let started = runs
        .start_def(&project, &def, &options_of(&ctx, &body, &args))
        .map_err(|e| refusal(&e))?;
    let rid = started.run.id;
    let tracker = link_when_printed(Arc::clone(&ctx.svc), runs, project, rid.clone());
    tokio::spawn(tracker);
    Ok(json_reply(200, &json!({ "run": rid })))
}

/// Registers the check run route.
pub fn register(r: &mut Router) {
    r.post("/api/p/:key/checks", start);
}
