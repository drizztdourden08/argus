//! `coord item add "<title>" [--desc] [--ac]... [--status s] [--for T] [--type t] [--parent I]
//! [--assignee who] [--stage s] [--field k=v]... [--force "<reason>"]
//! [--before|--after <I>|--top|--bottom]`.
//!
//! A call that names none of the typed flags files the plain item it always did. One that names
//! any goes through the `create` op, which checks it against the registry.

use argus_board::Board;
use argus_board::content::{Context, check};
use argus_board::json::display;
use argus_board::limits::require_text;
use argus_board::ops::{Check, OpResult, authorize, plain_needs_parent};
use argus_board::store::create_doc;
use serde_json::{Map, Value, json};

use super::STATUSES;
use super::edit::warnings;
use super::force::{attach, reason};
use super::images::{keep_all, picked};
use super::typed;
use crate::cli::{Options, Outcome};
use crate::verbs::work::crud::{
    apply, apply_result, find_doc, is_lead, move_op, op, owner_arg, stored_links, who_of,
};
use crate::verbs::work::place::rank_to;
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

/// A status: one of the five.
pub(super) fn check_status(value: &Value) -> Res<String> {
    match value.as_str().filter(|s| STATUSES.contains(s)) {
        Some(status) => Ok(status.to_owned()),
        None => Err(bad(format!(
            "status must be one of {}.",
            STATUSES.join(", ")
        ))),
    }
}

/// What the call asks for, checked before the board is read.
struct Draft {
    title: String,
    description: String,
    ac: Vec<String>,
    status: String,
    flags: Map<String, Value>,
    force: Option<String>,
}

fn criteria(r: &Run<'_>) -> Res<Vec<String>> {
    r.options
        .many("ac")
        .iter()
        .map(|text| require_text("ac", &text.to_json()))
        .collect()
}

fn initial_status(r: &Run<'_>) -> Res<String> {
    match r.options.json("status") {
        None => Ok("todo".to_owned()),
        Some(given) => check_status(&given),
    }
}

/// The description, checked by the rich text rule of the registry for every item, typed or not.
fn description(r: &Run<'_>) -> Res<String> {
    let cx = Context {
        images: &[],
        ids: None,
        threads: None,
        title_max: None,
    };
    let checked = check("description", &r.opt("desc"), &cx)?;
    Ok(checked.as_str().unwrap_or_default().to_owned())
}

fn draft_of(r: &Run<'_>) -> Res<Draft> {
    let title = require_text("title", &r.arg_json(0))?;
    let flags = typed::flags(r)?;
    Ok(Draft {
        title,
        description: description(r)?,
        ac: criteria(r)?,
        status: initial_status(r)?,
        flags,
        force: reason(r.options)?,
    })
}

/// The plain item of always: its thread, origin and numbered criteria.
fn plain(board: &mut Board, who: &Caller, owner: &str, draft: &Draft) -> Res<String> {
    plain_needs_parent(board)?;
    let ac: Vec<Value> = draft
        .ac
        .iter()
        .zip(1_u32..)
        .map(|(text, n)| json!({"n": n, "text": text, "done": false}))
        .collect();
    let from_other_lead = is_lead(who) && owner != who.thread;
    let fields = json!({
        "title": draft.title, "description": draft.description,
        "thread": owner, "status": draft.status,
        "origin": if from_other_lead { "lead" } else { "thread" },
        "ac": ac,
    });
    let fields = fields.as_object().cloned().unwrap_or_default();
    create_doc(board, "items", "I", fields, &who.actor)
}

/// A typed item given `--for T` is assigned to T too, unless the call names an assignee.
fn assign_for(set: &mut Map<String, Value>, options: &Options, owner: &str) {
    if options.text("for").is_some() && !set.contains_key("assignee") {
        set.insert("assignee".into(), json!(owner));
    }
}

fn created(extra: &Map<String, Value>) -> Res<String> {
    extra
        .get("created")
        .and_then(|c| c["id"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| bad("the item was not created."))
}

/// A typed item, made by the `create` op: the registry checks every value.
fn typed_item(
    board: &mut Board,
    who: &Caller,
    (owner, options): (&str, &Options),
    draft: Draft,
) -> Res<(String, OpResult)> {
    let mut set = Map::from_iter([
        ("title".to_owned(), json!(draft.title)),
        ("description".to_owned(), json!(draft.description)),
        ("ac".to_owned(), json!(draft.ac)),
        ("status".to_owned(), json!(draft.status)),
        ("thread".to_owned(), json!(owner)),
    ]);
    set.extend(draft.flags);
    stored_links(board, &mut set)?;
    assign_for(&mut set, options, owner);
    let rest = Map::from_iter([("set".to_owned(), Value::Object(set))]);
    let operation = attach(
        op(("create", "items", ""), rest, options)?,
        draft.force.as_deref(),
    );
    let res = apply_result(board, who, &operation)?;
    Ok((created(&res.extra)?, res))
}

fn file_at(options: &Options, board: &mut Board, who: &Caller, id: &str) -> Res<()> {
    if let Some(to) = rank_to(board, ("items", id), options)? {
        apply(board, who, &move_op("items", id, to, &Options::default())?)?;
    }
    Ok(())
}

/// `coord item add`.
pub(super) fn add(r: &mut Run<'_>) -> Outcome {
    let draft = draft_of(r)?;
    let pictures = picked(r, 1)?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let owner = owner_arg(board, &who, r.options)?;
        let check = Check::new("add", "items", None).to(Some(&owner));
        authorize(&who_of(&who), &check)?;
        let (id, res) = if draft.flags.is_empty() {
            (plain(board, &who, &owner, &draft)?, None)
        } else {
            let (id, res) = typed_item(board, &who, (owner.as_str(), r.options), draft)?;
            (id, Some(res))
        };
        keep_all(board, r, (id.as_str(), &who), &pictures)?;
        file_at(r.options, board, &who, &id)?;
        let made = find_doc(board, "items", Some(&id))?;
        let count = made["ac"].as_array().map_or(0, Vec::len);
        r.say(&[&format!(
            "{id}: {} ({}, {count} AC)",
            display(&made["title"]),
            display(&made["thread"])
        )]);
        if let Some(res) = res {
            warnings(r, &res);
        }
        Ok(())
    })
}
