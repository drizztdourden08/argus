//! `coord log rm <subject> <seq>`: removes one line. Someone else's line leaves an audit line and
//! a note to its author.

use argus_board::inbox::post;
use argus_board::json::{array_mut, as_integer, display, doc, doc_mut, set};
use argus_board::ops::{Check, authorize};
use argus_board::store::{LogLine, append_log, touch};
use argus_board::text::slug;
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use crate::cli::Outcome;
use crate::cli::out::clip;
use crate::verbs::work::crud::who_of;
use crate::verbs::work::run::{Run, bad};

const USAGE: &str = "usage: coord log rm <subject> <seq> (the #seq `coord log list` shows).";
const AUDIT: usize = 120;
const NOTE: usize = 240;

/// The line of a subject with this number.
fn find_line(board: &Value, subject: &str, seq: i64) -> Option<(usize, Value)> {
    let lines = doc(board, "log", subject)?["entries"].as_array()?;
    lines
        .iter()
        .position(|l| as_integer(l.get("seq")) == Some(seq) && l["seq"].is_number())
        .map(|at| (at, lines[at].clone()))
}

/// `coord log rm`.
pub(super) fn rm(r: &mut Run<'_>) -> Outcome {
    let subject = slug(r.arg(0).unwrap_or_default());
    let seq = r.arg(1).and_then(|s| as_integer(Some(&json!(s))));
    let (false, Some(seq)) = (subject.is_empty(), seq) else {
        return Err(bad(USAGE).into());
    };
    r.write(|board, r| {
        let who = r.begin(board)?;
        let Some((at, line)) = find_line(board, &subject, seq) else {
            return Err(bad(format!("no log line #{seq} under \"{subject}\".")));
        };
        authorize(&who_of(&who), &Check::new("rm", "log", Some(&line)))?;
        if let Some(entries) = doc_mut(board, "log", &subject) {
            array_mut(entries, "entries").remove(at);
            set(entries, "updatedAt", now_iso());
        }
        touch(board, "log", &subject);
        let author = display(&line["thread"]);
        if author != who.actor {
            let note = format!(
                "{} removed {subject} #{seq} ({author}): {}",
                who.actor,
                display(&line["text"])
            );
            let audit = clip(&note, AUDIT);
            append_log(
                board,
                "audit",
                &LogLine {
                    thread: &who.actor,
                    item: None,
                    text: &audit,
                },
            );
            if doc(board, "threads", &author).is_some_and(|t| !t.is_null()) {
                post(
                    board,
                    &author,
                    &who.actor,
                    ("note", &clip(&note, NOTE)),
                    None,
                );
            }
        }
        r.say(&[&format!("{subject} #{seq}: removed.")]);
        Ok(())
    })
}
