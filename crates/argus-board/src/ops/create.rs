//! The `create` op: a new item or action.

use argus_core::js;
use serde_json::{Map, Value, json};

use super::announce::{announce, coordinator_ids, refresh_needs_human};
use super::authorize::{Check, authorize};
use super::result::OpResult;
use super::target::{ITEM_STATUSES, id_arg, item_arg, one_of, thread_arg};
use super::who::{Role, Who};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, nullish, obj, shown};
use crate::limits::{optional_text, require_text, text_list};
use crate::store::{create_doc, rank_between};
use crate::text::{clip_text, slug};

/// The thread a created record belongs to: the one named, else the fallback, else the creator's
/// own thread, else (the owner creating without naming one) the first live coordinator.
fn owner_for(
    board: &Board,
    who: &Who,
    named: Option<&Value>,
    fallback: Option<String>,
) -> Result<String, BoardError> {
    let named = named.filter(|v| !v.is_null()).map(display);
    let id = named
        .or(fallback)
        .or_else(|| who.self_id().map(str::to_owned))
        .or_else(|| coordinator_ids(board, true).into_iter().next());
    let Some(id) = id else {
        return Err(refuse("name the thread this belongs to."));
    };
    thread_arg(board, &json!(id))
}

fn fields_of(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn item_fields(
    board: &Board,
    who: &Who,
    set: &Map<String, Value>,
) -> Result<(Map<String, Value>, String), BoardError> {
    let get = |key: &str| set.get(key).cloned().unwrap_or(Value::Null);
    let title = require_text("title", &get("title"))?;
    let owner = owner_for(board, who, set.get("thread"), None)?;
    let description = match get("description") {
        Value::String(s) if !js::trim(&s).is_empty() => require_text("description", &json!(s))?,
        _ => String::new(),
    };
    let thread_subject = doc(board, "threads", &owner).and_then(|t| nullish(t, "subject"));
    let subject = optional_text("subject", &get("subject"))?
        .or(thread_subject)
        .unwrap_or_else(|| "general".to_owned());
    let status = match set.get("status") {
        None => json!("todo"),
        Some(v) => one_of(&ITEM_STATUSES, v, "status")?,
    };
    let from_other_lead = who.role == Role::Coordinator && who.self_id() != Some(owner.as_str());
    let origin = if who.is_human() || from_other_lead {
        "coordinator"
    } else {
        "thread"
    };
    let ac: Vec<Value> = text_list("ac", &get("ac"), 20)?
        .into_iter()
        .zip(1_u32..)
        .map(|(text, n)| json!({"n": n, "text": text, "done": false}))
        .collect();
    let fields = json!({
        "title": title, "description": description, "subject": slug(&subject),
        "thread": owner, "status": status, "origin": origin, "ac": ac,
    });
    Ok((fields_of(fields), owner))
}

fn action_fields(
    board: &Board,
    who: &Who,
    set: &Map<String, Value>,
) -> Result<(Map<String, Value>, String), BoardError> {
    let body = require_text("action", set.get("text").unwrap_or(&Value::Null))?;
    let item = match set.get("item").filter(|v| js::truthy(v)) {
        Some(v) => Some(item_arg(board, v)?),
        None => None,
    };
    let item_thread = item
        .as_deref()
        .and_then(|i| doc(board, "items", i))
        .and_then(|i| nullish(i, "thread"));
    let owner = owner_for(board, who, set.get("thread"), item_thread)?;
    let fields = json!({
        "thread": owner, "item": item, "text": body, "status": "pending",
        "doneAt": null, "ackedAt": null,
    });
    Ok((fields_of(fields), owner))
}

/// Apply a `create` op: a new item (`I-n`) or action (`H-n`).
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let coll = op["target"]["c"].as_str();
    let empty = Map::new();
    let set = op["set"].as_object().unwrap_or(&empty);
    let (mut fields, owner, prefix) = match coll {
        Some("items") => {
            let (f, o) = item_fields(board, who, set)?;
            (f, o, "I")
        }
        Some("actions") => {
            let (f, o) = action_fields(board, who, set)?;
            (f, o, "H")
        }
        _ => {
            return Err(refuse(format!(
                "cannot create {} here: only items and actions.",
                shown(op["target"].get("c"))
            )));
        }
    };
    let coll = coll.unwrap_or_default();
    authorize(who, &Check::new("create", coll, None).to(Some(&owner)))?;
    let to = obj(&op["to"]);
    let before = to.get("before").and_then(id_arg);
    let after = to.get("after").and_then(id_arg);
    if before.is_some() || after.is_some() {
        let rank = rank_between(board, coll, before.as_deref(), after.as_deref(), None)?;
        fields.insert("rank".into(), js::number(rank));
    }
    let id = create_doc(board, coll, prefix, fields, &who.actor_of())?;
    if coll == "actions" {
        refresh_needs_human(board, &owner);
    }
    let created = doc(board, coll, &id).cloned().unwrap_or(Value::Null);
    let label = nullish(&created, "title")
        .or_else(|| nullish(&created, "text"))
        .unwrap_or_default();
    let said = format!("added {id} ({}) to {owner}", clip_text(&label, 50));
    let notify = announce(board, who, &[Some(owner)], &said, Some(&id));
    Ok(OpResult::applied(format!("{id} created"))
        .with_extra("created", json!({"c": coll, "id": id}))
        .with_notify(notify))
}
