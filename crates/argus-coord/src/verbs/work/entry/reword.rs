//! `coord entry set <E> ["<text>"] [--option "..."]... [--context "<c>"]`: rewords an entry (and
//! a question's choices). The author while it is open or pending; the coordinator at any time.

use argus_board::json::{display, doc_mut, set};
use argus_board::limits::{require_context, require_text, soft_context, text_list};
use argus_board::ops::{Check, authorize};
use argus_board::store::stamp;
use argus_core::clock::now_iso;
use serde_json::Value;

use super::{DECISION_OPTIONS, prefix_of};
use crate::cli::{Outcome, Val};
use crate::verbs::work::crud::{find_live, is_coordinator, who_of};
use crate::verbs::work::run::{Res, Run, bad};

/// The new context: a question or decision keeps the minimum, information is clipped.
fn new_context(r: &Run<'_>, kind: &str) -> Res<Option<Value>> {
    if r.text_opt("context").is_none() {
        return Ok(None);
    }
    let given = r.opt("context");
    if matches!(kind, "question" | "decision") {
        return Ok(Some(Value::from(require_context(&given)?)));
    }
    Ok(Some(soft_context(&given).map_or(Value::Null, Value::from)))
}

/// `coord entry set`.
pub(super) fn set_text(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let e = find_live(board, "entries", r.arg(0))?;
        let (id, kind) = (display(&e["id"]), display(&e["kind"]));
        if prefix_of(&kind).is_none() {
            return Err(bad(format!("{id} is not a text entry.")));
        }
        authorize(&who_of(&who), &Check::new("set", "entries", Some(&e)))?;
        let open = matches!(e["status"].as_str(), Some("open" | "pending"));
        if !is_coordinator(&who) && !open {
            return Err(bad(format!(
                "{id} is {}; only an open or pending entry can be reworded.",
                display(&e["status"])
            )));
        }
        let choices = r.options.many("option");
        if !choices.is_empty() && kind != "question" {
            return Err(bad("--option only applies to a question."));
        }
        let has_context = r.text_opt("context").is_some();
        if r.arg(1).is_none() && !has_context && choices.is_empty() {
            return Err(bad(format!("{kind} text is required.")));
        }
        let text = match r.arg(1) {
            None => e["text"].clone(),
            Some(given) => Value::from(require_text(&kind, &Value::from(given))?),
        };
        let counted = !(2..=5).contains(&choices.len());
        if !choices.is_empty() && e["needs"] == "decision" && counted {
            return Err(bad(format!("{DECISION_OPTIONS} Got {}.", choices.len())));
        }
        let given = Value::Array(choices.iter().map(Val::to_json).collect());
        let options = if choices.is_empty() {
            None
        } else {
            Some(text_list("option", &given, 5)?)
        };
        let context = new_context(r, &kind)?;
        if let Some(found) = doc_mut(board, "entries", &id) {
            set(found, "text", text);
            if let Some(options) = options {
                set(found, "options", options);
            }
            if let Some(context) = context {
                set(found, "context", context);
            }
            set(found, "editedAt", now_iso());
        }
        stamp(board, "entries", &id, &who.actor)?;
        r.say(&[&format!("{id}: reworded.")]);
        Ok(())
    })
}
