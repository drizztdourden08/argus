//! `coord action add "<t>" [--item I|T] [--for T] [--who owner|<thread>|lead:<key>]
//! [--before|--after <H>|--top|--bottom]`.

use argus_board::limits::require_text;
use argus_board::ops::{Check, authorize, refresh_needs_human};
use argus_board::store::create_doc;
use serde_json::{Value, json};

use super::{assignee_arg, live_record};
use crate::cli::{Options, Outcome};
use crate::verbs::work::crud::{apply, move_op, owner_arg, who_of};
use crate::verbs::work::place::rank_to;
use crate::verbs::work::run::Run;

/// `coord action add`.
pub(super) fn add(r: &mut Run<'_>) -> Outcome {
    let text = require_text("action", &r.arg_json(0))?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let owner = owner_arg(board, &who, r.options)?;
        let check = Check::new("add", "actions", None).to(Some(&owner));
        authorize(&who_of(&who), &check)?;
        let item = r.text_opt("item").map(str::to_owned);
        if let Some(id) = &item {
            live_record(board, id)?;
        }
        let assignee = assignee_arg(board, r)?;
        let fields = json!({
            "thread": owner, "item": item, "text": text, "status": "pending",
            "doneAt": null, "ackedAt": null,
        });
        let mut fields = fields.as_object().cloned().unwrap_or_default();
        if let Some(who_for) = &assignee {
            fields.insert("assignee".to_owned(), who_for.clone());
        }
        let id = create_doc(board, "actions", "H", fields, &who.actor)?;
        if let Some(to) = rank_to(board, ("actions", &id), r.options)? {
            apply(
                board,
                &who,
                &move_op("actions", &id, to, &Options::default())?,
            )?;
        }
        refresh_needs_human(board, &owner);
        r.say(&[&said(&id, &text, assignee.as_ref())]);
        Ok(())
    })
}

/// The confirmation: who the to-do waits for.
fn said(id: &str, text: &str, assignee: Option<&Value>) -> String {
    match assignee.and_then(Value::as_str).filter(|a| *a != "owner") {
        Some(who) => format!("{id} is for {who}: {text}"),
        None => format!("{id} needs the human: {text}"),
    }
}
