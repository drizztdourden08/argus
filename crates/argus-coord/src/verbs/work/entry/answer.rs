//! `coord answer <entry> "<text>" [--decision]`: a reply written from the CLI (the coordinator
//! relaying what the owner said in chat). The page writes the same shape through the tick.

use argus_board::inbox::post;
use argus_board::json::{display, doc_mut, set};
use argus_board::limits::require_text;
use argus_board::ops::{Check, authorize};
use argus_board::store::stamp;
use serde_json::json;

use super::create_entry;
use crate::cli::Outcome;
use crate::verbs::work::crud::{assert_live, find_doc, is_coordinator, who_of};
use crate::verbs::work::run::Run;

/// `coord answer`.
pub(super) fn answer(r: &mut Run<'_>) -> Outcome {
    let kind = if r.options.flag("decision") { "decision" } else { "followup" };
    let text = require_text(kind, &r.arg_json(1))?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let question = find_doc(board, "entries", r.arg(0))?;
        assert_live(&question, "entries")?;
        authorize(&who_of(&who), &Check::new("answer", "entries", Some(&question)))?;
        let author = if is_coordinator(&who) { "human" } else { who.thread.as_str() };
        let qid = display(&question["id"]);
        let fields = json!({
            "kind": kind, "item": question["item"], "thread": question["thread"],
            "author": author, "text": text, "replyTo": qid,
            "status": if kind == "decision" { "pending" } else { "info" },
        });
        let id = create_entry(board, fields, &who.actor)?;
        if matches!(question["kind"].as_str(), Some("question" | "message")) {
            if let Some(q) = doc_mut(board, "entries", &qid) {
                set(q, "status", "answered");
            }
            stamp(board, "entries", &qid, &who.actor)?;
        }
        let thread = display(&question["thread"]);
        if argus_core::js::truthy(&question["thread"]) {
            let line = format!("{id} on {qid}: {text}");
            post(board, &thread, author, (kind, &line), Some(&id));
        }
        r.say(&[&format!("{id} ({kind}) queued for {thread}.")]);
        Ok(())
    })
}
