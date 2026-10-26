//! `coord action set <H> [--text] [--item]`, `done <H> [--undo]` and
//! `move <H> [--to T] [--before|--after <H>|--top|--bottom]`.

use argus_board::json::{display, doc_mut, set as put};
use argus_board::limits::require_text;
use argus_board::ops::{Check, authorize, refresh_needs_human};
use argus_board::store::stamp;
use argus_core::clock::now_iso;
use serde_json::{Map, Value, json};

use crate::cli::Outcome;
use crate::verbs::work::crud::{
    apply, assert_live, find_doc, move_op, op, rev_of, thread_arg, who_of,
};
use crate::verbs::work::place::{has_placement, rank_to};
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
    if fields.is_empty() {
        return Err(bad("nothing to change: pass --text or --item.").into());
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let action = find_doc(board, "actions", r.arg(0))?;
        assert_live(&action, "actions")?;
        let id = display(&action["id"]);
        if let Some(item) = fields.get("item").and_then(Value::as_str).filter(|i| !i.is_empty()) {
            assert_live(&find_doc(board, "items", Some(item))?, "items")?;
        }
        authorize(&who_of(&who), &Check::new("set", "actions", Some(&action)))?;
        let names: Vec<&str> = fields.keys().map(String::as_str).collect();
        let rest = Map::from_iter([("set".to_owned(), Value::Object(fields.clone()))]);
        apply(board, &who, &op(("edit", "actions", &id), rest, r.options)?)?;
        let line = format!("{id}: {} updated (rev {}).", names.join(", "), rev_of(board, "actions", &id));
        r.say(&[&line]);
        Ok(())
    })
}

/// `coord action done`: done, or `--undo` to put it back to pending.
pub(super) fn done(r: &mut Run<'_>) -> Outcome {
    let undo = r.options.flag("undo");
    r.write(|board, r| {
        let who = r.begin(board)?;
        let action = find_doc(board, "actions", r.arg(0))?;
        assert_live(&action, "actions")?;
        authorize(&who_of(&who), &Check::new("done", "actions", Some(&action)))?;
        let id = display(&action["id"]);
        if let Some(found) = doc_mut(board, "actions", &id) {
            let done_at = if undo || !found["doneAt"].is_null() { found["doneAt"].clone() } else { json!(now_iso()) };
            put(found, "status", if undo { "pending" } else { "done" });
            put(found, "doneAt", if undo { Value::Null } else { done_at });
            put(found, "ackedAt", if undo { Value::Null } else { json!(now_iso()) });
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
    let mut to = Map::new();
    if let Some(thread) = r.text_opt("to") {
        to.insert("thread".into(), json!(thread));
    }
    if to.is_empty() && !has_placement(r.options) {
        return Err(bad("say where: --to <thread>, --before|--after <action>, --top or --bottom.").into());
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let action = find_doc(board, "actions", r.arg(0))?;
        assert_live(&action, "actions")?;
        let id = display(&action["id"]);
        let thread = to.get("thread").and_then(Value::as_str).map(str::to_owned);
        if let Some(name) = &thread {
            thread_arg(board, name)?;
        }
        let check = Check::new("move", "actions", Some(&action)).to(thread.as_deref());
        authorize(&who_of(&who), &check)?;
        if let Some(place) = rank_to(board, ("actions", &id), r.options)? {
            to.extend(place);
        }
        apply(board, &who, &move_op("actions", &id, to.clone(), r.options)?)?;
        let now = find_doc(board, "actions", Some(&id))?;
        refresh_needs_human(board, action["thread"].as_str().unwrap_or_default());
        refresh_needs_human(board, now["thread"].as_str().unwrap_or_default());
        r.say(&[&format!("{id}: {} (rev {}).", display(&now["thread"]), display(&now["rev"]))]);
        Ok(())
    })
}
