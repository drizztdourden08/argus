//! `coord log list [--subject <s>] [--n <count>]`.

use argus_board::json::{display, doc, section, to_number};
use argus_board::text::slug;
use serde_json::Value;

use crate::cli::Outcome;
use crate::cli::out::{ago, pad_end};
use crate::verbs::work::keys::js_keys;
use crate::verbs::work::run::Run;

const SHOWN: f64 = 20.0;

/// The first line of `lines.slice(-n)` out of `len`: a count taken from the end, a negative one
/// dropping from the start, a count that is not a number or is zero showing everything.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn tail_from(len: usize, n: f64) -> usize {
    let start = (-n).trunc();
    if start.is_nan() || start == 0.0 {
        return 0;
    }
    let steps = start.abs().min(1e15) as usize;
    if start < 0.0 { len.saturating_sub(steps) } else { steps.min(len) }
}

fn line(entry: &Value, now: i64) -> String {
    let seq = if entry["seq"].is_null() { "-".to_owned() } else { display(&entry["seq"]) };
    let item = entry["item"].as_str().filter(|i| !i.is_empty()).map_or_else(String::new, |i| format!("{i} "));
    format!(
        "  {} {} {} {item}{}",
        pad_end(&format!("#{seq}"), 6),
        pad_end(&ago(entry["ts"].as_str(), now), 8),
        pad_end(&display(&entry["thread"]), 14),
        display(&entry["text"])
    )
}

/// `coord log list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    let n = r.options.json("n").map_or(SHOWN, |v| to_number(Some(&v)));
    r.read(|board, r| {
        let subjects = match r.text_opt("subject") {
            Some(s) => vec![slug(s)],
            None => js_keys(section(board, "log").keys().cloned().collect()),
        };
        let now = r.now_ms();
        let mut lines: Vec<String> = Vec::new();
        for subject in subjects {
            let Some(found) = doc(board, "log", &subject).filter(|d| !d.is_null()) else {
                continue;
            };
            lines.push(format!("## {subject}"));
            let entries = found["entries"].as_array().map_or(&[][..], Vec::as_slice);
            lines.extend(entries[tail_from(entries.len(), n)..].iter().map(|e| line(e, now)));
        }
        if lines.is_empty() {
            r.say(&["log is empty."]);
        } else {
            r.say(&[&lines.join("\n")]);
        }
        Ok(())
    })
}
