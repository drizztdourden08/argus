//! The words of the alerts; the dwell keeps the stale words of `text.rs`.

use super::{Finding, Kind};
use crate::item_wakes::text::reminder_text;

const MAX_TEXT: usize = 300;
const MAX_LISTED: usize = 5;

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn head_of(kind: Kind, n: usize) -> String {
    let s = plural(n);
    match kind {
        Kind::Dwell => format!("{n} board item{s} stale: "),
        Kind::AssigneeGone => format!("{n} board item{s} lost the assignee: "),
        Kind::Unassigned => format!("{n} board item{s} without an assignee: "),
        Kind::OverLimit => format!("{n} Kanban column{s} over the limit: "),
    }
}

fn tail_of(kind: Kind) -> &'static str {
    match kind {
        Kind::Dwell => "",
        Kind::AssigneeGone => " Hand each item to a live thread or close it. Keep the board true.",
        Kind::Unassigned => " Give each item an assignee, or drop it. Keep the board true.",
        Kind::OverLimit => " Finish or move items out before starting more.",
    }
}

fn cells(list: &[Finding], shown: usize) -> String {
    let words: Vec<String> = list
        .iter()
        .take(shown)
        .map(|f| format!("{} {}", f.stale.id, f.note))
        .collect();
    let more = list.len() - shown;
    let rest = if more > 0 {
        format!(" (+{more} more)")
    } else {
        String::new()
    };
    format!("{}{rest}.", words.join(", "))
}

/// The line told for a group of one kind, cut until the whole text fits the 300 characters of a wake.
pub(in crate::item_wakes) fn alert_text(kind: Kind, list: &[Finding]) -> String {
    if kind == Kind::Dwell {
        let stale: Vec<_> = list.iter().map(|f| f.stale.clone()).collect();
        return reminder_text(&stale);
    }
    let head = head_of(kind, list.len());
    for shown in (1..=list.len().min(MAX_LISTED)).rev() {
        let text = format!("{head}{}{}", cells(list, shown), tail_of(kind));
        if text.chars().count() <= MAX_TEXT || shown == 1 {
            return text;
        }
    }
    head
}
