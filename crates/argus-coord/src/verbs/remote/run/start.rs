//! `coord run start | stop`: launch a defined run through the dashboard, and end one.

use argus_board::limits::optional_text;
use argus_board::store::read_board;
use serde_json::{Value, json};

use super::follow::{LogRead, exit_code_of, follow_logs, seconds, summary};
use super::names::{is_agent_id, need_name, need_rid};
use super::{Fail, Res};
use crate::cli::exit::CoordError;
use crate::cli::route::Call;
use crate::client::Client;
use crate::verbs::remote::support::link::project_of;
use crate::verbs::remote::support::scope::coord_home;
use crate::verbs::remote::support::text::cell;
use crate::who::caller;

/// Who is asking, for the run's `by`: the calling thread when there is one.
fn by_of(call: &Call<'_>) -> String {
    let board = read_board(&coord_home(call), project_of(call))
        .ok()
        .flatten();
    board
        .and_then(|b| caller(&b, &call.inv.cwd, &call.inv.env, &call.options).ok())
        .map_or_else(|| "cli".to_owned(), |who| who.actor)
}

fn start_body(call: &Call<'_>, name: &str) -> Result<Value, CoordError> {
    let mut body = json!({ "name": name, "by": by_of(call) });
    if call.options.get("agent").is_some() {
        match call.options.text("agent").filter(|a| is_agent_id(a)) {
            Some(agent) => body["agent"] = json!(agent),
            None => return Err(CoordError::refuse("--agent takes an agent id such as A-3.")),
        }
    }
    let reason = call.options.json("reason").unwrap_or(Value::Null);
    if let Some(reason) = optional_text("reason", &reason)? {
        body["reason"] = json!(reason);
    }
    if call.options.flag("visible") {
        body["visible"] = json!(true);
    }
    Ok(body)
}

/// `coord run start <name> [--agent A-n] [--reason "..."] [--wait] [--visible]`.
pub(super) fn start(call: &mut Call<'_>, client: &Client) -> Res<()> {
    let name = need_name(call.positional.first(), "name")?;
    let body = start_body(call, &name)?;
    let project = project_of(call).to_owned();
    let out = client.post(&format!("/api/p/{project}/runs"), &body)?;
    let already = if crate::verbs::remote::support::text::truthy(&out, "already") {
        " (already live, nothing new started)"
    } else {
        ""
    };
    let run = &out["run"];
    let head = format!(
        "{} {} {}{already}",
        cell(run.get("id")),
        cell(run.get("name")),
        cell(run.get("status"))
    );
    call.inv.say(&[&head]);
    if !call.options.flag("wait") {
        return Ok(());
    }
    let rid = cell(run.get("id"));
    let last = follow_logs(call, client, &rid, &LogRead::waiting())?;
    let began = last
        .get("startedAt")
        .filter(|v| !v.is_null())
        .unwrap_or(&last["queuedAt"]);
    let took = seconds(began, &last["endedAt"], call.inv.clock.now_ms());
    call.inv.warn(&format!("{} in {took}", summary(&last)));
    match exit_code_of(&last) {
        0 => Ok(()),
        code => Err(Fail::Coord(CoordError::Exit(code))),
    }
}

/// `coord run stop <R-n>`.
pub(super) fn stop(call: &mut Call<'_>, client: &Client) -> Res<()> {
    let rid = need_rid(call.positional.first())?;
    let project = project_of(call).to_owned();
    let answer = client.post(&format!("/api/p/{project}/runs/{rid}/stop"), &json!({}))?;
    call.inv.say(&[&summary(&answer["run"])]);
    Ok(())
}
