//! `coord agent set <A-id|name> [--task t] [--model m --why "..."] [--kind k] [--item I]
//! [--branch b] [--worktree w] [--status waiting|working] [--checkin m] [--claude-id id]`: changes
//! the named fields. A new model needs a new why; `--status` on an ended record is the resume of
//! the same sub-agent under its Claude id.

use argus_board::Board;
use argus_board::agents::is_live;
use argus_board::json::{display, doc, nullish, section, truthy};
use argus_board::store::stamp;
use serde_json::Value;

use super::bind::claude_id;
use super::fields::{Fields, add_model, checkin, claude_id_option, fields_of, model_why};
use super::find::{Want, agent_mut, clear_ending, named};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::authority::change_agent;
use crate::verbs::agent::base::opt::{given, value};
use crate::verbs::agent::base::{Run, write};
use crate::who::Caller;

const FIELDS: [&str; 5] = ["task", "kind", "item", "branch", "worktree"];

/// The model a record has or asked for: the new `--model`, else the one it was recorded with,
/// else the one its transcript ran on.
fn model_of(asked: Option<&str>, agent: &Value) -> Option<String> {
    asked
        .map(str::to_owned)
        .or_else(|| nullish(agent, "model"))
        .or_else(|| nullish(agent, "modelUsed"))
}

/// What a resume needs: the record to carry the Claude id.
fn check_resume(agent: &Value, id: &str, run: &Run<'_>) -> Result<(), CoordError> {
    if claude_id(agent).is_none() && claude_id_option(run.options)?.is_none() {
        return Err(CoordError::refuse(format!(
            "{id} is {}, not running: only a record with a Claude agent id can be reopened (the sub-agent resumed under that id); pass --claude-id <id> too, or start a new record.",
            display(&agent["status"])
        )));
    }
    Ok(())
}

fn add_justification(
    fields: &mut Fields,
    agent: &Value,
    (model, why): (Option<&str>, Option<&str>),
) -> Result<(), CoordError> {
    if model.is_none() && why.is_none() {
        return Ok(());
    }
    let named = model_of(model, agent)
        .filter(|m| !m.is_empty())
        .ok_or_else(|| {
            CoordError::refuse(
                "--why goes with --model: this agent has no model recorded, so pass both.",
            )
        })?;
    let kind = fields
        .get("kind")
        .map_or_else(|| display(&agent["kind"]), display);
    add_model(fields, model_why(Some(&named), why, &kind)?);
    Ok(())
}

fn add_claude_id(
    board: &Board,
    fields: &mut Fields,
    id: &str,
    run: &Run<'_>,
) -> Result<(), CoordError> {
    let Some(given) = claude_id_option(run.options)? else {
        return Ok(());
    };
    let other = section(board, "agents").values().find(|x| {
        display(&x["id"]) != id
            && !truthy(x, "deletedAt")
            && claude_id(x).as_deref() == Some(&given)
    });
    if let Some(other) = other {
        return Err(CoordError::refuse(format!(
            "{} already carries Claude agent id {given}: it is the same sub-agent, keep one record (coord agent done {id}).",
            display(&other["id"])
        )));
    }
    fields.insert("claudeId".to_owned(), Value::from(given));
    Ok(())
}

fn add_checkin(
    fields: &mut Fields,
    agent: &Value,
    model: Option<&str>,
    asked: Option<&str>,
) -> Result<(), CoordError> {
    if asked.is_none() && model.is_none() {
        return Ok(());
    }
    let named = model_of(model, agent);
    let kept = nullish(agent, "checkinMin").filter(|_| model.is_some());
    let raw = asked.map(str::to_owned).or(kept);
    if let Some(minutes) = checkin(named.as_deref(), raw.as_deref())? {
        fields.insert("checkinMin".to_owned(), Value::from(minutes));
    }
    Ok(())
}

fn add_status(
    fields: &mut Fields,
    agent: &Value,
    id: &str,
    status: Option<&str>,
) -> Result<(), CoordError> {
    let Some(status) = status else {
        return Ok(());
    };
    if agent["status"] == "stop-requested" {
        return Err(CoordError::refuse(format!(
            "the owner asked to stop {id}: stop it, then run coord agent stop {id}."
        )));
    }
    fields.insert("status".to_owned(), Value::from(status));
    Ok(())
}

/// What the call changes on the record, every field checked.
fn plan(
    board: &Board,
    id: &str,
    run: &Run<'_>,
    status: Option<&str>,
) -> Result<Fields, CoordError> {
    let agent = doc(board, "agents", id).cloned().unwrap_or(Value::Null);
    let mut fields = fields_of(board, run.options, &FIELDS)?;
    let model = value(run.options, "model")?;
    let why = value(run.options, "why")?;
    add_justification(&mut fields, &agent, (model, why))?;
    add_claude_id(board, &mut fields, id, run)?;
    add_checkin(&mut fields, &agent, model, value(run.options, "checkin")?)?;
    add_status(&mut fields, &agent, id, status)?;
    Ok(fields)
}

fn change(
    board: &mut Board,
    who: &Caller,
    run: &mut Run<'_>,
    status: Option<&str>,
) -> Result<(), CoordError> {
    let want = if status.is_some() || given(run.options, "why") {
        Want::Open
    } else {
        Want::Live
    };
    let id = named(board, who, run, want)?;
    let agent = agent_mut(board, &id)?;
    let hidden = truthy(agent, "archivedAt") || truthy(agent, "deletedAt");
    let reopening = status.is_some() && !is_live(agent) && !hidden;
    if status.is_some() && !is_live(agent) && !reopening {
        let why = if truthy(agent, "deletedAt") {
            "in the trash"
        } else {
            "archived"
        };
        return Err(CoordError::refuse(format!("{id} is {why}, not running.")));
    }
    if reopening {
        check_resume(agent, &id, run)?;
    }
    change_agent(who, agent)?;
    let fields = plan(board, &id, run, status)?;
    let agent = agent_mut(board, &id)?;
    if reopening {
        clear_ending(agent);
    }
    if fields.is_empty() {
        return Err(CoordError::refuse(
            "nothing to change: pass --task, --model with --why, --kind, --item, --branch, --worktree, --status, --claude-id or --checkin (opus-level agents).",
        ));
    }
    let keys: Vec<&str> = fields.keys().map(String::as_str).collect();
    let shown = keys.join(", ");
    for (key, v) in fields {
        argus_board::json::set(agent, &key, v);
    }
    stamp(board, "agents", &id, &who.actor)?;
    let rev = doc(board, "agents", &id).map_or_else(String::new, |a| display(&a["rev"]));
    run.say(&format!("{id}: {shown} updated (rev {rev})."));
    Ok(())
}

/// `coord agent set`.
///
/// # Errors
/// A value a field refuses, the model rules, nothing to change, no such agent.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let status = value(&call.options, "status")?.map(str::to_owned);
    if status
        .as_deref()
        .is_some_and(|s| s != "waiting" && s != "working")
    {
        return Err(CoordError::refuse(
            "--status must be waiting or working (done, fail and stop have their own verbs).",
        ));
    }
    write(call, |board, who, run| {
        change(board, who, run, status.as_deref())
    })
}
