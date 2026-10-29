//! `coord check [<check>] [--commit <sha>] [--pr PR-n] [--job gate|upgrade|all]
//! [--trigger push|pr|manual|release]`: starts a check run (`brock ci`) on the run engine of the
//! dashboard and prints its run `R-n` and, once Brock has written the record, its check `C-n`.
//! `coord check` with none of those is the board checker (`coord check [--stale] [--json]`).

use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::support::link::{connect, project_of};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::client::{Client, ClientError};
use crate::verbs::thread::run_check;
use crate::who::caller;

/// The options that ask for a check run.
const ASKS: [&str; 4] = ["commit", "pr", "job", "trigger"];
/// How long the record of a started run is waited for.
const LINK_WAIT: Duration = Duration::from_secs(30);
const POLL: Duration = Duration::from_millis(250);

fn asks_for_a_run(call: &Call<'_>) -> bool {
    !call.positional.is_empty() || ASKS.iter().any(|o| call.options.get(o).is_some())
}

fn by_of(call: &Call<'_>) -> String {
    let home = call.inv.env.paths(&call.inv.cwd).coord_home;
    let board = argus_board::store::read_board(&home, project_of(call))
        .ok()
        .flatten();
    board
        .and_then(|b| caller(&b, &call.inv.cwd, &call.inv.env, &call.options).ok())
        .map_or_else(|| "cli".to_owned(), |who| who.actor)
}

/// The request: the check word and the options typed, each a text.
fn body_of(call: &Call<'_>) -> Result<Value, CoordError> {
    let mut body = json!({ "by": by_of(call) });
    if let Some(check) = call.positional.first() {
        body["check"] = json!(check);
    }
    for name in ASKS {
        if call.options.get(name).is_some() {
            let Some(value) = call.options.text(name) else {
                return Err(CoordError::refuse(format!("--{name} takes a value.")));
            };
            body[name] = json!(value);
        }
    }
    Ok(body)
}

/// A failed call: a stopped dashboard is exit 3 with the plain message, as for `coord run`.
fn fail(call: &mut Call<'_>, e: ClientError) -> CoordError {
    if matches!(e, ClientError::Down(_)) {
        call.inv.warn(&format!("coord: {e}"));
        return CoordError::Exit(3);
    }
    e.into()
}

fn record_of(client: &Client, project: &str, rid: &str) -> Option<String> {
    let answer = client
        .get(&format!("/api/p/{project}/checks?run={rid}"))
        .ok()?;
    answer["runs"][0]["id"].as_str().map(str::to_owned)
}

fn ended(client: &Client, project: &str, rid: &str) -> Option<String> {
    let answer = client.get(&format!("/api/p/{project}/runs/{rid}")).ok()?;
    let status = answer["run"]["status"].as_str()?;
    (status != "queued" && status != "running").then(|| status.to_owned())
}

/// Waits for the record the run prints first: its `C-n`, or why there is none.
fn wait_for_record(client: &Client, project: &str, rid: &str) -> Result<String, String> {
    let deadline = Instant::now() + LINK_WAIT;
    loop {
        if let Some(id) = record_of(client, project, rid) {
            return Ok(id);
        }
        if let Some(status) = ended(client, project, rid) {
            return record_of(client, project, rid).ok_or(format!("the run {status}"));
        }
        if Instant::now() >= deadline {
            return Err("none yet".to_owned());
        }
        std::thread::sleep(POLL);
    }
}

fn start(call: &mut Call<'_>) -> Outcome {
    let body = body_of(call)?;
    let client = connect(call, true).map_err(|e| fail(call, e))?;
    let project = project_of(call).to_owned();
    let started = client.post(&format!("/api/p/{project}/checks"), &body);
    let answer = started.map_err(|e| fail(call, e))?;
    let rid = answer["run"].as_str().unwrap_or_default().to_owned();
    let line = match wait_for_record(&client, &project, &rid) {
        Ok(id) => format!("{rid} {id} check started"),
        Err(why) => {
            format!("{rid} check started (no record: {why}; coord run logs {rid} shows the output)")
        }
    };
    call.inv.say(&[&line]);
    Ok(())
}

/// `coord check`: a check run when asked for one, else the board checker.
///
/// # Errors
/// A refusal, or the dashboard not running (exit 3).
pub fn handle(call: &mut Call<'_>) -> Outcome {
    if asks_for_a_run(call) {
        start(call)
    } else {
        run_check(call)
    }
}
