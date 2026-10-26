//! `coord action add "<t>" [--item I] [--for T] [--before|--after <H>|--top|--bottom]`.

use argus_board::limits::require_text;
use argus_board::ops::{Check, authorize, refresh_needs_human};
use argus_board::store::create_doc;
use serde_json::json;

use crate::cli::Outcome;
use crate::verbs::work::crud::{apply, assert_live, find_doc, move_op, owner_arg, who_of};
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
            assert_live(&find_doc(board, "items", Some(id))?, "items")?;
        }
        let fields = json!({
            "thread": owner, "item": item, "text": text, "status": "pending",
            "doneAt": null, "ackedAt": null,
        });
        let fields = fields.as_object().cloned().unwrap_or_default();
        let id = create_doc(board, "actions", "H", fields, &who.actor)?;
        if let Some(to) = rank_to(board, ("actions", &id), r.options)? {
            apply(board, &who, &move_op("actions", &id, to, &Default::default())?)?;
        }
        refresh_needs_human(board, &owner);
        r.say(&[&format!("{id} needs the human: {text}")]);
        Ok(())
    })
}
