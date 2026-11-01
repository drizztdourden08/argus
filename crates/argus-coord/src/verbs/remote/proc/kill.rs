//! `coord proc kill | kill-rule`: end a process the monitor lists, or every process a rule takes.
//! Both are the Overlord's; the dashboard checks it again. Any other thread's `kill` is the kill
//! of its own project's process ([`super::own`]).

use argus_core::js;
use serde_json::{Value, json};

use super::dashboard;
use super::list::mem;
use super::own;
use super::patch::older_than;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::list_of;
use crate::verbs::remote::support::overlord;
use crate::verbs::remote::support::text::{cell, collapse, is_integer, number_of};

const ADVICE: &str = "Ask it with coord lead portfolio \"<what and why>\", or the owner uses the buttons on Argus > Processes.";
const KILL_USAGE: &str = "name the process: coord proc kill <pid> [--tree] [--reason \"...\"]";

fn pid_of(call: &Call<'_>) -> Result<i64, CoordError> {
    let n = call.positional.first().map_or(f64::NAN, |p| number_of(p));
    #[allow(clippy::cast_possible_truncation)]
    if is_integer(n) && n > 0.0 {
        Ok(n as i64)
    } else {
        Err(CoordError::refuse(KILL_USAGE))
    }
}

fn reason_of(call: &Call<'_>) -> Result<String, CoordError> {
    let reason = call
        .options
        .text("reason")
        .map(collapse)
        .unwrap_or_default();
    if js::len(&reason) < 5 {
        return Err(CoordError::refuse(
            "--reason is required: say why it has to go (it goes into the activity log).",
        ));
    }
    Ok(reason)
}

/// `coord proc kill <pid> [--tree] [--reason "..."]`: the Overlord's kill of a listed process
/// (`--reason` required), else the calling thread's kill of its own process.
pub(super) fn kill(call: &mut Call<'_>) -> Outcome {
    let pid = pid_of(call)?;
    let Ok(by) = overlord::actor(call, "kill a process", ADVICE) else {
        return own::kill(call, pid);
    };
    let reason = reason_of(call)?;
    let client = dashboard(call)?;
    let listing = client.get("/api/procs")?;
    let Some(found) = list_of(&listing, "procs")
        .iter()
        .find(|p| p["pid"].as_i64() == Some(pid))
    else {
        return Err(CoordError::refuse(format!(
            "process {pid} is not in the listing (gone, or no rule takes it): coord proc list"
        )));
    };
    let tree = call.options.flag("tree");
    let mut body = json!({ "pid": pid });
    if let Some(started) = found.get("startedAt") {
        body["startedAt"] = started.clone();
    }
    body["tree"] = json!(tree);
    body["reason"] = json!(reason);
    body["by"] = json!(by);
    let out = client.post("/api/procs/kill", &body)?;
    if !js::truthy(&out["killed"]) {
        return Err(CoordError::refuse(format!(
            "could not kill {} {pid}: {}",
            cell(out.get("name")),
            cell(out.get("error"))
        )));
    }
    let and_tree = if tree { " and its tree" } else { "" };
    let line = format!(
        "killed {} {pid}{and_tree} ({}).",
        cell(out.get("name")),
        mem(&found["memMB"])
    );
    call.inv.say(&[&line]);
    Ok(())
}

fn listed(items: &[Value]) -> String {
    let named: Vec<String> = items
        .iter()
        .map(|x| format!("{} {}", cell(x.get("name")), cell(x.get("pid"))))
        .collect();
    named.join(", ")
}

fn failures(failed: &[Value]) -> String {
    let named: Vec<String> = failed
        .iter()
        .map(|x| {
            format!(
                "{} {}: {}",
                cell(x.get("name")),
                cell(x.get("pid")),
                cell(x.get("error"))
            )
        })
        .collect();
    named.join("; ")
}

fn report(rule: &str, out: &Value) -> String {
    let killed = list_of(out, "killed");
    let failed = list_of(out, "failed");
    let names = if killed.is_empty() {
        String::new()
    } else {
        format!(" ({})", listed(killed))
    };
    let broken = if failed.is_empty() {
        String::new()
    } else {
        format!(", failed {} ({})", failed.len(), failures(failed))
    };
    format!(
        "{rule}: killed {}{names}{broken}, skipped {}.",
        killed.len(),
        list_of(out, "skipped").len()
    )
}

/// `coord proc kill-rule <rule> [--older-than 6h] [--idle-only] [--reason "..."]`.
pub(super) fn kill_rule(call: &mut Call<'_>) -> Outcome {
    let Some(rule) = call.positional.first().filter(|r| !r.is_empty()).cloned() else {
        return Err(CoordError::refuse(
            "name the rule: coord proc kill-rule <rule> [--older-than 6h] [--idle-only]",
        ));
    };
    let by = overlord::actor(call, "kill processes", ADVICE)?;
    let reason = call
        .options
        .text("reason")
        .map_or(Value::Null, |r| json!(r));
    let mut body = json!({
        "rule": rule, "by": by, "idleOnly": call.options.flag("idle-only"), "reason": reason,
    });
    if let Some(minutes) = older_than(&call.options)? {
        body["olderThanMin"] = minutes;
    }
    let out = dashboard(call)?.post("/api/procs/kill-rule", &body)?;
    call.inv.say(&[&report(&rule, &out)]);
    if list_of(&out, "failed").is_empty() {
        Ok(())
    } else {
        Err(CoordError::Exit(1))
    }
}
