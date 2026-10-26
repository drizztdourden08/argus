//! `coord log "<what just happened>" [--subject <s>] [--item <I>]`.

use argus_board::json::{doc, map_mut, nullish};
use argus_board::limits::{optional_text, require_text};
use argus_board::store::{LogLine, append_log};
use argus_board::text::slug;

use crate::cli::Outcome;
use crate::verbs::work::keys::js_order;
use crate::verbs::work::run::{Run, bad};

/// `coord log`: one line under a subject.
pub(super) fn write(r: &mut Run<'_>) -> Outcome {
    let text = require_text("log", &r.arg_json(0))?;
    r.write(|board, r| {
        let me = r.begin(board)?.thread;
        let item = r.text_opt("item").map(str::to_owned);
        if let Some(id) = item.as_deref().filter(|i| !i.is_empty()) {
            if doc(board, "items", id).is_none_or(serde_json::Value::is_null) {
                return Err(bad(format!("no item \"{id}\".")));
            }
        }
        let from_item = doc(board, "items", item.as_deref().unwrap_or_default())
            .and_then(|i| nullish(i, "subject"));
        let from_thread = doc(board, "threads", &me).and_then(|t| nullish(t, "subject"));
        let subject = optional_text("subject", &r.opt("subject"))?
            .or(from_item)
            .or(from_thread)
            .unwrap_or_else(|| "general".to_owned());
        let subject = slug(&subject);
        let line = LogLine { thread: &me, item: item.as_deref(), text: &text };
        append_log(board, &subject, &line);
        js_order(map_mut(board, "log"));
        r.say(&[&format!("[{subject}] {me}: {text}")]);
        Ok(())
    })
}
