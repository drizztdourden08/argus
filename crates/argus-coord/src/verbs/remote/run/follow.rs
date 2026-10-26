//! The log of a run: paged from the dashboard, printed as it comes, followed until the run is
//! over, and the one-line summary of a run.

use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use argus_board::json::{display, shown};
use argus_core::clock::parse_ms;
use serde_json::Value;

use super::Res;
use super::names::{int_option, need_rid};
use crate::cli::route::Call;
use crate::client::Client;
use crate::verbs::remote::support::link::{list_of, project_of};
use crate::verbs::remote::support::text::{cell, round};

const POLL: Duration = Duration::from_millis(400);
const PAGE: usize = 1000;
const FINAL: [&str; 3] = ["exited", "failed", "stopped"];

/// How a log is read.
pub(super) struct LogRead {
    after: String,
    follow: bool,
    with_sys: bool,
}

impl LogRead {
    /// Until the run is over, the dashboard's own notes left out (`start --wait`).
    pub(super) fn waiting() -> Self {
        LogRead {
            after: "0".to_owned(),
            follow: true,
            with_sys: false,
        }
    }
}

fn is_final(run: &Value) -> bool {
    run["status"].as_str().is_some_and(|s| FINAL.contains(&s))
}

/// `seconds(from, to)`: how long a run took, `45s` or `3m`.
pub(super) fn seconds(from: &Value, to: &Value, now_ms: i64) -> String {
    if !argus_core::js::truthy(from) {
        return String::new();
    }
    let start = parse_ms(&display(from));
    let end = if argus_core::js::truthy(to) {
        parse_ms(&display(to))
    } else {
        Some(now_ms)
    };
    let (Some(start), Some(end)) = (start, end) else {
        return "NaNm".to_owned();
    };
    #[allow(clippy::cast_precision_loss)]
    let secs = round(((end - start) as f64 / 1000.0).max(0.0));
    if secs < 90.0 {
        format!("{secs}s")
    } else {
        format!("{}m", round(secs / 60.0))
    }
}

/// The exit code a run gives `start --wait`.
pub(super) fn exit_code_of(run: &Value) -> i32 {
    match run["status"].as_str() {
        Some("exited") => 0,
        Some("stopped") => 130,
        _ => run["exitCode"]
            .as_i64()
            .filter(|code| *code != 0)
            .and_then(|code| i32::try_from(code).ok())
            .unwrap_or(1),
    }
}

/// `<id> <name> <status> (code n): <error>`.
pub(super) fn summary(run: &Value) -> String {
    let stopped = run["status"] == "stopped";
    let code = match run.get("exitCode") {
        Some(Value::Null) => String::new(),
        _ if stopped => String::new(),
        code => format!(" (code {})", shown(code)),
    };
    let error = if argus_core::js::truthy(&run["error"]) {
        format!(": {}", cell(run.get("error")))
    } else {
        String::new()
    };
    format!(
        "{} {} {}{code}{error}",
        cell(run.get("id")),
        cell(run.get("name")),
        cell(run.get("status"))
    )
}

/// Prints a line as it comes; the dashboard's own notes only when asked for.
fn print_line(call: &mut Call<'_>, line: &Value, with_sys: bool) {
    let text = shown(line.get("text"));
    match line["stream"].as_str() {
        Some("out") => writeln!(call.inv.stdout, "{text}").ok(),
        Some("err") => writeln!(call.inv.stderr, "{text}").ok(),
        _ if with_sys => writeln!(call.inv.stderr, "# {text}").ok(),
        _ => None,
    };
}

fn print_page(call: &mut Call<'_>, page: &Value, with_sys: bool) {
    for line in list_of(page, "lines") {
        print_line(call, line, with_sys);
    }
}

/// Pages the log from `after`; when following, keeps polling until the run is over. Returns the
/// run as it ended.
pub(super) fn follow_logs(
    call: &mut Call<'_>,
    client: &Client,
    rid: &str,
    how: &LogRead,
) -> Res<Value> {
    let project = project_of(call).to_owned();
    let base = format!("/api/p/{project}/runs/{rid}");
    let mut cursor = how.after.clone();
    loop {
        let page = client.get(&format!("{base}/logs?after={cursor}&limit={PAGE}"))?;
        print_page(call, &page, how.with_sys);
        cursor = display(&page["next"]);
        if list_of(&page, "lines").len() >= PAGE {
            continue;
        }
        let run = client.get(&base)?["run"].clone();
        if !is_final(&run) && how.follow {
            sleep(POLL);
            continue;
        }
        if is_final(&run) && how.follow {
            let rest = client.get(&format!("{base}/logs?after={cursor}&limit=5000"))?;
            print_page(call, &rest, how.with_sys);
        }
        return Ok(run);
    }
}

/// `coord run logs <R-n> [--follow] [--tail n]`.
pub(super) fn logs(call: &mut Call<'_>, client: &Client) -> Res<()> {
    let rid = need_rid(call.positional.first())?;
    let project = project_of(call).to_owned();
    let mut after = "0".to_owned();
    if call.options.get("tail").is_some() {
        let n = int_option(&call.options, "tail", 200, (1, 5000))?;
        let tail = client.get(&format!("/api/p/{project}/runs/{rid}/logs?tail={n}"))?;
        print_page(call, &tail, true);
        after = display(&tail["next"]);
        if crate::verbs::remote::support::text::truthy(&tail, "dropped") {
            let dropped = cell(tail.get("dropped"));
            call.inv.warn(&format!(
                "# {dropped} lines were dropped from the middle of this log (size cap)"
            ));
        }
    }
    let follow = call.options.flag("follow");
    let how = LogRead {
        after,
        follow,
        with_sys: true,
    };
    let run = follow_logs(call, client, &rid, &how)?;
    if follow {
        call.inv.warn(&summary(&run));
    }
    Ok(())
}
