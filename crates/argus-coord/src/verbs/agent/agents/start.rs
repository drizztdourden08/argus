//! `coord agent start <name> "<task>" --model m --why "..." --item I-n [--item ...] (--worktree
//! <path|auto> | --master) [--blocked-by id ...] [--checkin 15..30]`: registers a running sub-agent
//! and prints its A-id. An agent works on an item and runs somewhere, so a start without both is
//! refused with the fix. A record the hooks already made for it is filled in instead of adding a
//! second one, and a same-named agent still running is marked stopped.

use argus_agents::model_rules::ModelWhy;
use argus_agents::text::same_model;
use argus_board::Board;
use argus_board::agents::{is_live, live_agent_ids};
use argus_board::json::{display, doc, section, set, truthy};
use argus_board::limits::require_str;
use argus_board::store::{create_doc, stamp};
use argus_board::text::slug;
use argus_core::clock::now_iso;
use serde_json::{Map, Value};

use super::bind::{claude_id, hook_record, launch_for};
use super::fields::{Fields, add_model, checkin, claude_id_option, fields_of, model_why};
use super::find::{agent_mut, clear_ending, owner_of};
use super::links::{for_items, for_start};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::apply::allow_create;
use crate::verbs::agent::base::opt::{value, word};
use crate::verbs::agent::base::{Run, write};
use crate::who::Caller;

/// What a registration is made of.
struct Entry<'a> {
    name: &'a str,
    task: &'a str,
    owner: String,
    fields: Fields,
    model: ModelWhy,
    checkin: Option<u32>,
    claude_id: Option<String>,
    /// The words the hook records and launch notes are matched against.
    text: String,
}

/// The `; replaced A-1 (same name, now stopped)` tail, after stopping the agents of that name.
fn replace_running(
    board: &mut Board,
    who: &Caller,
    entry: &Entry<'_>,
    keep: Option<&str>,
) -> Result<String, CoordError> {
    let replaced: Vec<String> = live_agent_ids(board, Some(&entry.owner))
        .into_iter()
        .filter(|id| {
            doc(board, "agents", id).is_some_and(|a| a["name"].as_str() == Some(entry.name))
                && Some(id.as_str()) != keep
        })
        .collect();
    for id in &replaced {
        let agent = agent_mut(board, id)?;
        set(agent, "status", "stopped");
        set(agent, "finishedAt", now_iso());
        stamp(board, "agents", id, &who.actor)?;
    }
    Ok(if replaced.is_empty() {
        String::new()
    } else {
        format!(
            "; replaced {} (same name, now stopped)",
            replaced.join(", ")
        )
    })
}

/// The hook's record is this agent: it keeps its id, source and transcript, the lead's fields go
/// on top. Returns the line to print.
fn bind(
    board: &mut Board,
    who: &Caller,
    entry: &Entry<'_>,
    id: &str,
) -> Result<String, CoordError> {
    let agent = agent_mut(board, id)?;
    let resumed = entry.claude_id.is_some()
        && !is_live(agent)
        && !truthy(agent, "archivedAt")
        && !truthy(agent, "deletedAt");
    set(agent, "name", entry.name);
    set(agent, "task", entry.task);
    for (key, v) in &entry.fields {
        set(agent, key, v.clone());
    }
    let mut model = Fields::new();
    add_model(&mut model, entry.model.clone());
    for (key, v) in model {
        set(agent, &key, v);
    }
    if let Some(minutes) = entry.checkin {
        set(agent, "checkinMin", minutes);
    }
    set(agent, "leadBound", true);
    if resumed {
        set(agent, "status", "working");
        clear_ending(agent);
    }
    if !truthy(agent, "claudeId") {
        let hook = agent["hookId"].clone();
        match (&entry.claude_id, truthy(agent, "hookId")) {
            (Some(given), _) => set(agent, "claudeId", given.as_str()),
            (None, true) => set(agent, "claudeId", hook),
            (None, false) => {}
        }
    }
    if truthy(agent, "modelUsed") {
        let same = same_model(&entry.model.model, &display(&agent["modelUsed"]));
        set(agent, "modelMismatch", !same);
    }
    let item = for_items(agent);
    stamp(board, "agents", id, &who.actor)?;
    let running = live_agent_ids(board, Some(&entry.owner)).len();
    let how = if resumed {
        "resumed: the closed record of this Claude id is running again"
    } else {
        "the hook had recorded it, bound to that record"
    };
    Ok(format!(
        "{id}: agent {} started on {}{item} ({running} running; {how})",
        entry.name, entry.owner
    ))
}

/// A new record, with the defaults of a fresh agent. Returns the line to print.
fn add(board: &mut Board, who: &Caller, entry: &Entry<'_>) -> Result<String, CoordError> {
    let mut base = Map::new();
    for (key, v) in [
        ("name", Value::from(entry.name)),
        ("thread", Value::from(entry.owner.as_str())),
        ("task", Value::from(entry.task)),
        ("model", Value::Null),
        ("modelWhy", Value::Null),
        ("kind", Value::from("build")),
        ("item", Value::Null),
        ("items", Value::Array(Vec::new())),
        ("branch", Value::Null),
        ("worktree", Value::Null),
        ("master", Value::Bool(false)),
        ("blockedBy", Value::Array(Vec::new())),
        ("status", Value::from("working")),
        ("note", Value::Null),
        ("result", Value::Null),
        ("startedAt", Value::from(now_iso())),
        ("finishedAt", Value::Null),
    ] {
        base.insert(key.to_owned(), v);
    }
    if let Some(id) = &entry.claude_id {
        base.insert("claudeId".to_owned(), Value::from(id.as_str()));
    }
    base.extend(entry.fields.clone());
    add_model(&mut base, entry.model.clone());
    if let Some(minutes) = entry.checkin {
        base.insert("checkinMin".to_owned(), Value::from(minutes));
    }
    let id = create_doc(board, "agents", "A", base, &who.actor)?;
    let item = doc(board, "agents", &id).map(for_items).unwrap_or_default();
    let running = live_agent_ids(board, Some(&entry.owner)).len();
    Ok(format!(
        "{id}: agent {} started on {}{item} ({running} running)",
        entry.name, entry.owner
    ))
}

/// The id of the record the Claude id names, whatever its age or state.
fn by_claude_id(board: &Board, id: &str) -> Option<String> {
    section(board, "agents")
        .values()
        .find(|a| claude_id(a).as_deref() == Some(id) && !truthy(a, "deletedAt"))
        .map(|a| display(&a["id"]))
}

/// `fields.<key> ?? ''`.
fn text_of<'a>(fields: &'a Fields, key: &str) -> &'a str {
    fields.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn entry_of<'a>(
    board: &Board,
    run: &Run<'_>,
    (name, task): (&'a str, &'a str),
    who: &Caller,
) -> Result<Entry<'a>, CoordError> {
    let owner = owner_of(board, Some(who), run.options)?;
    allow_create(who, "agents", &owner)?;
    let mut fields = fields_of(run.options, &["kind", "branch"])?;
    let kind = fields
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("build");
    let model = model_why(
        value(run.options, "model")?,
        value(run.options, "why")?,
        kind,
    )?;
    let checkin = checkin(Some(&model.model), value(run.options, "checkin")?)?;
    fields.extend(for_start(board, run.options)?);
    let text = format!(
        "{name} {task} {} {}",
        text_of(&fields, "branch"),
        text_of(&fields, "worktree")
    );
    let claude_id =
        claude_id_option(run.options)?.or_else(|| launch_for(board, &owner, &text, run.now_ms));
    Ok(Entry {
        name,
        task,
        owner,
        fields,
        model,
        checkin,
        claude_id,
        text,
    })
}

fn register(
    board: &mut Board,
    who: &Caller,
    run: &mut Run<'_>,
    words: (&str, &str),
) -> Result<(), CoordError> {
    let entry = entry_of(board, run, words, who)?;
    let hooked = match entry.claude_id.as_deref() {
        Some(id) => by_claude_id(board, id),
        None => hook_record(board, &entry.owner, &entry.text, run.now_ms),
    };
    let swapped = replace_running(board, who, &entry, hooked.as_deref())?;
    let line = match &hooked {
        Some(id) => bind(board, who, &entry, id)?,
        None => add(board, who, &entry)?,
    };
    run.say(&format!("{line}{swapped}"));
    Ok(())
}

/// `coord agent start`.
///
/// # Errors
/// A refusal of the name, the task, the model rules or the caller.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let name = slug(&require_str("name", word(&call.positional, 0))?);
    if name.is_empty() {
        return Err(CoordError::refuse(
            "agent name must contain letters or digits.",
        ));
    }
    let task = require_str("task", word(&call.positional, 1))?;
    write(call, |board, who, run| {
        register(board, who, run, (&name, &task))
    })
}
