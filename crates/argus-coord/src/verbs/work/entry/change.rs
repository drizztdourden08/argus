//! `coord entry pick <E>`, `done <E>` and `move <E> --item <I>`. A decision is picked only by the
//! thread it was routed to (or the lead); the routing is never rewritten, so nobody takes
//! another thread's decision.

use argus_board::json::{display, doc_mut, set};
use argus_board::ops::{Check, authorize};
use argus_board::store::stamp;
use serde_json::Value;

use super::item_arg;
use crate::cli::Outcome;
use crate::verbs::work::crud::{checked, find_live, who_of};
use crate::verbs::work::run::{Run, bad};

fn set_status(board: &mut argus_board::Board, id: &str, status: &str) {
    if let Some(e) = doc_mut(board, "entries", id) {
        set(e, "status", status);
    }
}

/// `coord entry pick`: starts a pending decision.
pub(super) fn pick(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let e = checked(board, &who, ("entries", "pick"), r.arg(0))?;
        let id = display(&e["id"]);
        if e["kind"] != "decision" || e["status"] != "pending" {
            let was = if e["kind"] == "decision" {
                display(&e["status"])
            } else {
                format!("a {}", display(&e["kind"]))
            };
            return Err(bad(format!(
                "{id} is {was}, only a pending decision can be picked."
            )));
        }
        set_status(board, &id, "doing");
        stamp(board, "entries", &id, &who.actor)?;
        r.say(&[&format!("{id}: doing ({})", who.thread)]);
        Ok(())
    })
}

/// `coord entry done`: closes a question, dismisses a notice or message, finishes the rest.
pub(super) fn done(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let e = checked(board, &who, ("entries", "done"), r.arg(0))?;
        let id = display(&e["id"]);
        let status = match e["kind"].as_str() {
            Some("question") => "closed",
            Some("notice" | "message") => "dismissed",
            _ => "done",
        };
        set_status(board, &id, status);
        stamp(board, "entries", &id, &who.actor)?;
        r.say(&[&format!("{id}: {status}")]);
        Ok(())
    })
}

/// `coord entry move`: a question or decision to another item (your own to your own item).
pub(super) fn move_to(r: &mut Run<'_>) -> Outcome {
    let Some(target) = r.text_opt("item").map(str::to_owned) else {
        return Err(bad("say the item: coord entry move <E> --item <I>.").into());
    };
    r.write(|board, r| {
        let who = r.begin(board)?;
        let e = find_live(board, "entries", r.arg(0))?;
        let item = item_arg(board, Some(&target))?;
        let check = Check::new("move", "entries", Some(&e)).to(item["thread"].as_str());
        authorize(&who_of(&who), &check)?;
        let (id, item_id) = (display(&e["id"]), display(&item["id"]));
        if let Some(found) = doc_mut(board, "entries", &id) {
            set(found, "item", Value::from(item_id.clone()));
        }
        stamp(board, "entries", &id, &who.actor)?;
        r.say(&[&format!("{id}: now on {item_id}.")]);
        Ok(())
    })
}
