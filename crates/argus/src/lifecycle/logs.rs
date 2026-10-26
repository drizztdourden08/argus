//! `argus logs [--lines n] [--follow] [--run R-n --project <key>]`: the tail of the server log
//! file, or of a run's log through the server. `--follow` keeps printing until the run ends (or,
//! for the server log, until the command is interrupted).

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

use argus_core::Paths;
use argus_core::ids::is_key;
use serde_json::{Value, json};
use tokio::time::sleep;

use super::live::{Live, api, up};

const DEFAULT_LINES: usize = 50;
const RUN_POLL: Duration = Duration::from_millis(700);
const FILE_POLL: Duration = Duration::from_millis(500);

/// What the caller asked for.
pub struct Request<'a> {
    /// `--lines`, as typed.
    pub lines: Option<&'a str>,
    /// `--follow`.
    pub follow: bool,
    /// `--run`.
    pub run: Option<&'a str>,
    /// `--project`.
    pub project: Option<&'a str>,
}

fn is_run_id(text: &str) -> bool {
    text.strip_prefix("R-")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn print_lines(page: &Value) {
    for line in page["lines"].as_array().into_iter().flatten() {
        let text = line["text"].as_str().unwrap_or_default();
        println!("{}", text.strip_suffix('\n').unwrap_or(text));
    }
}

fn finished(run: &Value) -> bool {
    !matches!(run["status"].as_str(), Some("running" | "queued"))
}

async fn follow_run(at: &Live, base: &str, mut page: Value) -> Result<(), String> {
    loop {
        sleep(RUN_POLL).await;
        let after = page["next"].clone();
        page = api(at, "GET", &format!("{base}/logs?after={after}"), &json!({})).await?;
        print_lines(&page);
        let run = api(at, "GET", base, &json!({})).await?;
        if finished(&run["run"]) && page["lines"].as_array().is_none_or(Vec::is_empty) {
            return Ok(());
        }
    }
}

async fn run_log(paths: &Paths, n: usize, request: &Request<'_>) -> Result<u8, String> {
    let (Some(run), Some(project)) = (request.run, request.project.filter(|p| is_key(p))) else {
        eprintln!("dashboard: logs --run R-n needs --project <key>");
        return Ok(2);
    };
    if !is_run_id(run) {
        eprintln!("dashboard: logs --run R-n needs --project <key>");
        return Ok(2);
    }
    let Some(running) = up(paths).await else {
        eprintln!("dashboard is not running");
        return Ok(1);
    };
    let base = format!("/api/p/{project}/runs/{run}");
    let page = api(
        &running.at,
        "GET",
        &format!("{base}/logs?tail={n}"),
        &json!({}),
    )
    .await?;
    print_lines(&page);
    if request.follow {
        follow_run(&running.at, &base, page).await?;
    }
    Ok(0)
}

fn tail(file: &Path, n: usize) -> Vec<String> {
    let text = fs::read_to_string(file).unwrap_or_default();
    let lines: Vec<&str> = text.split('\n').filter(|l| !l.is_empty()).collect();
    lines[lines.len().saturating_sub(n)..]
        .iter()
        .map(|l| (*l).to_owned())
        .collect()
}

/// New text after `offset`, with the offset it ends at; a file that shrank starts over.
fn grown(file: &Path, offset: u64) -> Option<(String, u64)> {
    let size = fs::metadata(file).ok()?.len();
    let from = if size < offset { 0 } else { offset };
    if size == from {
        return Some((String::new(), size));
    }
    let mut handle = File::open(file).ok()?;
    handle.seek(SeekFrom::Start(from)).ok()?;
    let mut bytes = Vec::new();
    handle.take(size - from).read_to_end(&mut bytes).ok()?;
    Some((String::from_utf8_lossy(&bytes).into_owned(), size))
}

async fn server_log(paths: &Paths, n: usize, follow: bool) -> Result<u8, String> {
    let file = paths.logs.join("server.log");
    for line in tail(&file, n) {
        println!("{line}");
    }
    if !follow {
        return Ok(0);
    }
    let mut offset = fs::metadata(&file).map_or(0, |m| m.len());
    loop {
        sleep(FILE_POLL).await;
        if let Some((text, end)) = grown(&file, offset) {
            print!("{text}");
            offset = end;
        }
    }
}

/// `--lines` as typed: the default when absent, a positive integer otherwise.
fn lines_of(text: Option<&str>) -> Option<usize> {
    text.map_or(Some(DEFAULT_LINES), |t| {
        t.parse::<usize>().ok().filter(|n| *n >= 1)
    })
}

/// Prints the log. A `--follow` of the server log ends only when the command is interrupted.
///
/// # Errors
///
/// A call to the server failed.
pub async fn logs(paths: &Paths, request: &Request<'_>) -> Result<u8, String> {
    let Some(n) = lines_of(request.lines) else {
        eprintln!("dashboard: --lines must be a positive integer");
        return Ok(2);
    };
    if request.run.is_some() {
        return run_log(paths, n, request).await;
    }
    server_log(paths, n, request.follow).await
}
