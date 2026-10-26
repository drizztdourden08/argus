//! `coord ask <item> "<q>" --context "<c>" [--decide] [--option "<c>"]...`,
//! `coord decide <item> "<d>" --context "<c>"`, `coord notify "<t>" [--item I] [--level l]
//! [--context "<c>"]` and `coord message "<t>" [--item I] [--context "<c>"]`.

use argus_board::limits::{require_context, require_text, soft_context, text_list};
use serde_json::{Value, json};

use super::{DECISION_OPTIONS, create_entry, item_arg, on_item};
use crate::cli::{Outcome, Val};
use crate::verbs::work::run::{Run, bad};

const LEVELS: [&str; 3] = ["info", "warn", "bad"];

fn choices_of(given: &[Val]) -> Value {
    Value::Array(given.iter().map(Val::to_json).collect())
}

/// `coord ask`: a question, or a decision request when it carries options.
pub(super) fn ask(r: &mut Run<'_>) -> Outcome {
    let text = require_text("question", &r.arg_json(1))?;
    let context = require_context(&r.opt("context"))?;
    let given = r.options.many("option");
    let decision = r.options.flag("decide") || !given.is_empty();
    if decision && !(2..=5).contains(&given.len()) {
        return Err(bad(format!("{DECISION_OPTIONS} Got {}.", given.len())).into());
    }
    let choices = text_list("option", &choices_of(&given), 5)?;
    let needs = if decision { "decision" } else { "answer" };
    r.write(|board, r| {
        let (who, item) = on_item(board, r)?;
        let fields = json!({
            "kind": "question", "item": item["id"], "thread": who.thread, "author": who.thread,
            "text": text, "context": context, "needs": needs, "options": choices, "status": "open",
        });
        let id = create_entry(board, &fields, &who.actor)?;
        let kind = if decision {
            " (decision request, shown in the feed)"
        } else {
            ""
        };
        let item_id = argus_board::json::display(&item["id"]);
        r.say(&[
            &format!("{id} asked on {item_id}{kind}: {text}"),
            "The answer reaches your inbox; hooks hand it to you on your next turn.",
        ]);
        Ok(())
    })
}

/// `coord decide`: a decision the thread records as done.
pub(super) fn decide(r: &mut Run<'_>) -> Outcome {
    let text = require_text("decision", &r.arg_json(1))?;
    let context = require_context(&r.opt("context"))?;
    r.write(|board, r| {
        let (who, item) = on_item(board, r)?;
        let fields = json!({
            "kind": "decision", "item": item["id"], "thread": who.thread, "author": who.thread,
            "text": text, "context": context, "status": "done",
        });
        let id = create_entry(board, &fields, &who.actor)?;
        let item_id = argus_board::json::display(&item["id"]);
        r.say(&[&format!("{id} recorded on {item_id}: {text}")]);
        Ok(())
    })
}

/// `coord message overlord "text"` once posted the word "overlord" as the whole message: these
/// two verbs take one text and go to the owner only, so a second word means another recipient.
fn single_text(r: &Run<'_>, verb: &str) -> Result<Value, crate::cli::CoordError> {
    if r.args.len() > 1 {
        return Err(bad(format!(
            "coord {verb} takes one quoted text and goes to the OWNER (got {} arguments: quote the whole text). For another project's lead use coord lead <project> \"<text>\"; for a thread of this board coord inbox send <thread> \"<text>\".",
            r.args.len()
        ))
        .into());
    }
    Ok(r.arg_json(0))
}

fn feed_item(board: &argus_board::Board, r: &Run<'_>) -> crate::verbs::work::run::Res<Value> {
    match r.text_opt("item") {
        Some(reference) => Ok(item_arg(board, Some(reference))?["id"].clone()),
        None => Ok(Value::Null),
    }
}

/// `coord notify`: a notice for the feed.
pub(super) fn notify(r: &mut Run<'_>) -> Outcome {
    let text = require_text("notice", &single_text(r, "notify")?)?;
    let level = r.text_opt("level").unwrap_or("info").to_owned();
    if !LEVELS.contains(&level.as_str()) {
        return Err(bad(format!("level must be one of {}.", LEVELS.join(", "))).into());
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let fields = json!({
            "kind": "notice", "item": feed_item(board, r)?, "thread": who.thread,
            "author": who.thread, "text": text, "context": soft_context(&r.opt("context")),
            "level": level, "status": "open",
        });
        let id = create_entry(board, &fields, &who.actor)?;
        r.say(&[&format!("{id} posted to the feed: {text}")]);
        Ok(())
    })
}

/// `coord message`: a message for the owner.
pub(super) fn message(r: &mut Run<'_>) -> Outcome {
    let text = require_text("message", &single_text(r, "message")?)?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let fields = json!({
            "kind": "message", "item": feed_item(board, r)?, "thread": who.thread,
            "author": who.thread, "text": text, "context": soft_context(&r.opt("context")),
            "status": "open",
        });
        let id = create_entry(board, &fields, &who.actor)?;
        r.say(&[&format!("{id} sent to the owner: {text}")]);
        Ok(())
    })
}
