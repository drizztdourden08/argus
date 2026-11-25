//! Starting a check run (`docs/local-remote.md`): `POST /api/p/:key/checks {commit?, pr?, check?,
//! job?, trigger?, localpkg?}` runs `brock ci` in the root of the project on the run engine (hidden, logs
//! kept, Stop and history are the run engine's) and answers `{run: "R-n"}`. When the run prints
//! its first line, the path of Brock's record, the pair is kept and `GET /api/p/:key/checks`
//! shows `run` on that record. The guard (`check_guard`) reads the tracked changes of the folders
//! the run can touch before it starts and after it ends; a run that changed tracked files is
//! failed, on its run record and its `C-n` entry, with one warn line in the server log.

use std::sync::Arc;
use std::time::Duration;

use argus_board::json::doc;
use argus_core::clock::now_iso;
use argus_core::log::Logger;
use argus_core::remote::ci_root_of;
use argus_core::{ApiError, ErrorCode};
use argus_launchers::{LogOptions, Origin, RunDef, RunKind, RunService, StartOptions, check_key};
use argus_store::Db;
use argus_worktrees::path_key;
use serde_json::{Value, json};

use super::blocking::blocking;
use super::check_guard::Before;
use super::params::key;
use super::runs::{refusal, service};
use super::write::{actor, object_body, outward};
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
    localpkg: bool,
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

fn flag_of(body: &Value, name: &str) -> Result<bool, ApiError> {
    match &body[name] {
        Value::Null => Ok(false),
        Value::Bool(on) => Ok(*on),
        _ => Err(ApiError::bad_request(format!(
            "{name} must be true or false"
        ))),
    }
}

fn commit_text(sha: &str) -> bool {
    sha.bytes().all(|b| b.is_ascii_hexdigit()) && (7..=40).contains(&sha.len())
}

fn commit_of(body: &Value) -> Result<Option<String>, ApiError> {
    let Some(sha) = text(body, "commit")? else {
        return Ok(None);
    };
    if commit_text(sha) {
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
            localpkg: flag_of(body, "localpkg")?,
        })
    }

    /// A pull request without a commit is checked at the head commit it records, the one
    /// `coord pr merge` looks a check run up by. An explicit commit wins; an unknown pull
    /// request is a 404.
    fn at_head_of_pr(mut self, ctx: &Ctx, project: &str) -> Result<Args, ApiError> {
        let (Some(pr), None) = (&self.pr, &self.commit) else {
            return Ok(self);
        };
        let board = ctx.svc.boards().board(project);
        let held = board.as_ref().and_then(|b| doc(b, "prs", pr));
        let Some(held) = held else {
            let sentence = format!("no pull request \"{pr}\" on {project}.");
            return Err(ApiError::new(ErrorCode::NotFound, sentence));
        };
        let recorded = held["headCommit"].as_str().map(str::to_ascii_lowercase);
        self.commit = recorded.filter(|c| commit_text(c));
        Ok(self)
    }

    /// `<program> ci [check] [--commit c] [--pr p] [--job j] [--trigger t] [--localpkg] --project key`.
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
        if self.localpkg {
            words.push("--localpkg".to_owned());
        }
        words.extend(["--project".to_owned(), project.to_owned()]);
        words.join(" ")
    }

    fn reason(&self) -> String {
        let of = self.pr.as_ref().or(self.commit.as_ref());
        format!("check {}", of.map_or("the working tree", String::as_str))
    }
}

/// The root of the project from the boards and the config.
fn project_root(ctx: &Ctx, project: &str) -> Option<String> {
    let known = ctx.svc.boards().projects();
    known.into_iter().find(|p| p.key == project)?.root
}

/// The folder `brock ci` runs in: the integration checkout of `coord remote ci-root` when one
/// is set, else the root of the project.
fn root_of(ctx: &Ctx, project: &str) -> Result<String, ApiError> {
    let root = ci_root_of(&ctx.svc.paths.coord_home, project)
        .or_else(|| project_root(ctx, project))
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
fn keep_pair(db: &Arc<Db>, log: &Logger, (project, rid): (&str, &str), line: &str) {
    let Some(run_id) = run_id_of(project, line) else {
        return;
    };
    let links = CheckLinks::new(Arc::clone(db));
    if !links.link(project, &run_id, rid) {
        let detail = json!({ "run": rid, "record": run_id });
        log.warn("check run not linked to its record", detail);
    }
}

/// The last link pass of every check run, made by the run engine once the log of the run is
/// whole and before the run is reported ended: the record is linked and the check index reads
/// the record, however fast the run ended, so whoever sees the run ended finds its `C-n`.
pub(crate) fn link_on_finish(svc: &Services, runs: &RunService<RunHost>) {
    let (db, log) = (Arc::clone(&svc.db), Arc::clone(&svc.log));
    let checks = svc.records.as_ref().map(|r| Arc::clone(&r.checks));
    runs.on_finish(move |runs, row| {
        if row.name != NAME {
            return;
        }
        if let Some(line) = first_line(runs, &row.scope, &row.id) {
            keep_pair(&db, &log, (&row.scope, &row.id), &line);
        }
        if let Some(checks) = &checks {
            checks.rescan();
        }
    });
}

/// The folders the guard watches: the one the run works in, and the project root when that
/// differs.
fn watched(cwd: &str, root: Option<String>) -> Vec<String> {
    let same = |r: &String| path_key(r, cfg!(windows)) == path_key(cwd, cfg!(windows));
    let mut dirs = vec![cwd.to_owned()];
    dirs.extend(root.filter(|r| !same(r)));
    dirs
}

/// Fails a run that changed tracked files: its run record, its `C-n` entry when its first line
/// named one, and one warn line. Blocking: git runs.
fn judge(
    svc: &Services,
    runs: &RunService<RunHost>,
    (project, rid): (&str, &str),
    before: &Before,
) {
    let Some(note) = before.verdict() else {
        return;
    };
    let failed = runs.fail_ended(project, rid, &note).is_some();
    let record = first_line(runs, project, rid).and_then(|line| run_id_of(project, &line));
    let links = CheckLinks::new(Arc::clone(&svc.db));
    let flagged = record
        .as_deref()
        .is_some_and(|id| links.flag(project, id, &note));
    let detail = json!({ "project": project, "run": rid, "record": record, "note": note,
        "runFailed": failed, "recordFailed": flagged });
    svc.log.warn("check run changed tracked files", detail);
}

/// Waits for the run to end. False when the server stops first.
async fn ended(svc: &Services, runs: &RunService<RunHost>, project: &str, rid: &str) -> bool {
    while live(runs, project, rid) {
        tokio::select! {
            () = tokio::time::sleep(POLL) => {}
            () = svc.stop.cancelled() => return false,
        }
    }
    true
}

/// Follows a check run: links its record when it prints it, then judges it when it ends.
async fn follow(
    svc: Arc<Services>,
    runs: Arc<RunService<RunHost>>,
    (project, rid): (String, String),
    before: Before,
) {
    link_when_printed(
        Arc::clone(&svc),
        Arc::clone(&runs),
        project.clone(),
        rid.clone(),
    )
    .await;
    if !ended(&svc, &runs, &project, &rid).await {
        return;
    }
    let verdict = move || judge(&svc, &runs, (&project, &rid), &before);
    let _ = tokio::task::spawn_blocking(verdict).await;
}

/// Links the record as soon as the first line is there, so a running check shows its `C-n`
/// early (`link_on_finish` makes sure of it at the end). Ends with the run, with the first line,
/// or with the server.
async fn link_when_printed(
    svc: Arc<Services>,
    runs: Arc<RunService<RunHost>>,
    project: String,
    rid: String,
) {
    loop {
        if let Some(line) = first_line(&runs, &project, &rid) {
            keep_pair(&svc.db, &svc.log, (&project, &rid), &line);
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

/// Check runs start only when `checks.enabled` is `true` in `config.json`: paused by default
/// since `brock ci` once removed tracked files of a main checkout.
fn paused(ctx: &Ctx) -> Result<(), ApiError> {
    let config = ctx.svc.config();
    let on = config
        .extra
        .get("checks")
        .and_then(|c| c["enabled"].as_bool());
    if on == Some(true) {
        return Ok(());
    }
    Err(ApiError::new(
        ErrorCode::Conflict,
        "check runs are paused (set checks.enabled to true in config.json once brock ci is safe)"
            .to_owned(),
    ))
}

async fn start(ctx: Ctx) -> Reply {
    outward(&ctx)?;
    paused(&ctx)?;
    let runs = service(&ctx)?;
    let body = object_body(&ctx).await?;
    let project = key(&ctx)?;
    check_key(&project).map_err(|e| refusal(&e))?;
    let args = Args::parse(&body)?.at_head_of_pr(&ctx, &project)?;
    let cmd = args.command(&program_of(&ctx), &project);
    let cwd = root_of(&ctx, &project)?;
    let dirs = watched(&cwd, project_root(&ctx, &project));
    let before = blocking(move || Before::take(dirs)).await?;
    let def = definition(cmd, cwd);
    let started = runs
        .start_def(&project, &def, &options_of(&ctx, &body, &args))
        .map_err(|e| refusal(&e))?;
    let rid = started.run.id;
    let tracker = follow(Arc::clone(&ctx.svc), runs, (project, rid.clone()), before);
    tokio::spawn(tracker);
    Ok(json_reply(200, &json!({ "run": rid })))
}

/// Registers the check run route.
pub fn register(r: &mut Router) {
    r.post("/api/p/:key/checks", start);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K1: the `localpkg` flag of a check run reaches the `brock ci` command line.
    use super::*;

    #[test]
    fn localpkg_true_adds_the_flag_before_the_project() {
        let args = Args::parse(&json!({ "job": "gate", "localpkg": true })).expect("args");
        assert_eq!(
            args.command("brock", "alpha"),
            "brock ci --job gate --localpkg --project alpha"
        );
    }

    #[test]
    fn localpkg_missing_or_false_leaves_the_command_as_it_was() {
        for body in [json!({}), json!({ "localpkg": false })] {
            let args = Args::parse(&body).expect("args");
            assert_eq!(args.command("brock", "alpha"), "brock ci --project alpha");
        }
    }

    #[test]
    fn localpkg_that_is_not_a_bool_is_refused() {
        assert!(Args::parse(&json!({ "localpkg": "yes" })).is_err());
    }
}
