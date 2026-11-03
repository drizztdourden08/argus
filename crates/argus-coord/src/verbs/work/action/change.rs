//! `coord action set <H> [--text] [--item] [--who]`, `done <H> [--undo]` (the assignee may) and
//! `move <H> [--to T] [--before|--after <H>|--top|--bottom]`.

use argus_board::json::{display, doc_mut, set as put};
use argus_board::limits::require_text;
use argus_board::ops::{Check, authorize, refresh_needs_human};
use argus_board::store::stamp;
use argus_core::clock::now_iso;
use serde_json::{Map, Value, json};

use super::{assignee_of, live_record};
use crate::cli::Outcome;
use crate::verbs::work::crud::{
    apply, find_doc, find_live, move_doc, move_target, op, rev_of, who_of,
};
use crate::verbs::work::place::has_placement;
use crate::verbs::work::run::{Run, bad};

/// `coord action set`.
pub(super) fn set(r: &mut Run<'_>) -> Outcome {
    let mut fields = Map::new();
    if let Some(text) = r.text_opt("text") {
        fields.insert("text".into(), json!(require_text("action", &json!(text))?));
    }
    if let Some(item) = r.text_opt("item") {
        fields.insert("item".into(), json!(item));
    }
    if let Some(who) = r.text_opt("who") {
        fields.insert("assignee".into(), json!(who));
    }
    if fields.is_empty() {
        return Err(bad("nothing to change: pass --text or --item.").into());
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let action = find_live(board, "actions", r.arg(0))?;
        let id = display(&action["id"]);
        if let Some(item) = fields
            .get("item")
            .and_then(Value::as_str)
            .filter(|i| !i.is_empty())
        {
            live_record(board, item)?;
        }
        authorize(&who_of(&who), &Check::new("set", "actions", Some(&action)))?;
        let names: Vec<&str> = fields.keys().map(String::as_str).collect();
        let rest = Map::from_iter([("set".to_owned(), Value::Object(fields.clone()))]);
        apply(board, &who, &op(("edit", "actions", &id), rest, r.options)?)?;
        let line = format!(
            "{id}: {} updated (rev {}).",
            names.join(", "),
            rev_of(board, "actions", &id)
        );
        r.say(&[&line]);
        Ok(())
    })
}

/// `coord action done`: done, or `--undo` to put it back to pending.
pub(super) fn done(r: &mut Run<'_>) -> Outcome {
    let undo = r.options.flag("undo");
    r.write(|board, r| {
        let who = r.begin(board)?;
        let action = find_live(board, "actions", r.arg(0))?;
        if assignee_of(&action) != who.thread {
            authorize(&who_of(&who), &Check::new("done", "actions", Some(&action)))?;
        }
        let id = display(&action["id"]);
        if let Some(found) = doc_mut(board, "actions", &id) {
            let done_at = if undo || !found["doneAt"].is_null() {
                found["doneAt"].clone()
            } else {
                json!(now_iso())
            };
            put(found, "status", if undo { "pending" } else { "done" });
            put(found, "doneAt", if undo { Value::Null } else { done_at });
            put(
                found,
                "ackedAt",
                if undo { Value::Null } else { json!(now_iso()) },
            );
        }
        stamp(board, "actions", &id, &who.actor)?;
        refresh_needs_human(board, action["thread"].as_str().unwrap_or_default());
        let state = if undo { "pending again" } else { "done" };
        r.say(&[&format!("{id}: {state} (by {})", who.thread)]);
        Ok(())
    })
}

/// `coord action move`.
pub(super) fn move_to(r: &mut Run<'_>) -> Outcome {
    let to = move_target(r.options);
    if to.is_empty() && !has_placement(r.options) {
        return Err(
            bad("say where: --to <thread>, --before|--after <action>, --top or --bottom.").into(),
        );
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let (before, id) = move_doc(board, &who, ("actions", r.arg(0)), (to, r.options))?;
        let now = find_doc(board, "actions", Some(&id))?;
        refresh_needs_human(board, before["thread"].as_str().unwrap_or_default());
        refresh_needs_human(board, now["thread"].as_str().unwrap_or_default());
        r.say(&[&format!(
            "{id}: {} (rev {}).",
            display(&now["thread"]),
            display(&now["rev"])
        )]);
        Ok(())
    })
}
