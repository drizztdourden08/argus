//! `coord item add "<title>" [--desc] [--ac]... [--subject] [--status s] [--for T]
//! [--before|--after <I>|--top|--bottom]`.

use argus_board::json::{nullish, doc};
use argus_board::limits::{optional_text, require_text};
use argus_board::ops::{Check, authorize};
use argus_board::store::create_doc;
use argus_board::text::slug;
use serde_json::{Map, Value, json};

use super::STATUSES;
use crate::cli::Outcome;
use crate::verbs::work::crud::{apply, move_op, owner_arg, who_of};
use crate::verbs::work::place::rank_to;
use crate::verbs::work::run::{Res, Run, bad};

/// A status: one of the five.
pub(super) fn check_status(value: &Value) -> Res<String> {
    match value.as_str().filter(|s| STATUSES.contains(s)) {
        Some(status) => Ok(status.to_owned()),
        None => Err(bad(format!("status must be one of {}.", STATUSES.join(", ")))),
    }
}

fn criteria(r: &Run<'_>) -> Res<Vec<Value>> {
    r.options
        .many("ac")
        .iter()
        .zip(1_u32..)
        .map(|(text, n)| {
            let text = require_text("ac", &text.to_json())?;
            Ok(json!({"n": n, "text": text, "done": false}))
        })
        .collect()
}

fn initial_status(r: &Run<'_>) -> Res<String> {
    match r.options.json("status") {
        None => Ok("todo".to_owned()),
        Some(given) => check_status(&given),
    }
}

/// `coord item add`.
pub(super) fn add(r: &mut Run<'_>) -> Outcome {
    let title = require_text("title", &r.arg_json(0))?;
    let description = optional_text("description", &r.opt("desc"))?.unwrap_or_default();
    let ac = criteria(r)?;
    let status = initial_status(r)?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let owner = owner_arg(board, &who, r.options)?;
        let check = Check::new("add", "items", None).to(Some(&owner));
        authorize(&who_of(&who), &check)?;
        let thread_subject = doc(board, "threads", &owner).and_then(|t| nullish(t, "subject"));
        let subject = optional_text("subject", &r.opt("subject"))?
            .or(thread_subject)
            .unwrap_or_else(|| "general".to_owned());
        let from_other_lead = super::super::crud::is_coordinator(&who) && owner != who.thread;
        let fields = json!({
            "title": title, "description": description, "subject": slug(&subject),
            "thread": owner, "status": status,
            "origin": if from_other_lead { "coordinator" } else { "thread" },
            "ac": ac,
        });
        let fields = fields.as_object().cloned().unwrap_or_else(Map::new);
        let id = create_doc(board, "items", "I", fields, &who.actor)?;
        if let Some(to) = rank_to(board, ("items", &id), r.options)? {
            apply(board, &who, &move_op("items", &id, to, &Default::default())?)?;
        }
        r.say(&[&format!("{id}: {title} ({owner}, {} AC)", ac.len())]);
        Ok(())
    })
}
