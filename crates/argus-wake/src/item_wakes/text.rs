//! The words of the Kanban reminders.

use argus_core::clock::age_text;

use super::stale::Stale;

const MAX_TEXT: usize = 300;
const MAX_LISTED: usize = 5;
const DAY_MS: i64 = 86_400_000;
const TAIL: &str = " Per item: start it (coord item set <I-n> --status doing, then dispatch), update it (a note or a status) or drop or close it (coord item status <I-n> dropped|done). Keep the board true.";

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn cell(index: usize, s: &Stale) -> String {
    let age = age_text(s.quiet);
    if index == 0 {
        format!("{} {} {age} with no change", s.id, s.status)
    } else {
        format!("{} {} {age}", s.id, s.status)
    }
}

/// `3 board items stale: I-12 doing 5h with no change, I-15 todo 2d, I-18 review 9h (+1 more). ...`;
/// the engine cuts at 300, so the list shrinks until the whole text fits.
#[must_use]
pub fn reminder_text(list: &[Stale]) -> String {
    let head = format!("{} board item{} stale: ", list.len(), plural(list.len()));
    for n in (1..=list.len().min(MAX_LISTED)).rev() {
        let more = if list.len() > n {
            format!(" (+{} more)", list.len() - n)
        } else {
            String::new()
        };
        let cells: Vec<String> = list
            .iter()
            .take(n)
            .enumerate()
            .map(|(i, s)| cell(i, s))
            .collect();
        let text = format!("{head}{}{more}.{TAIL}", cells.join(", "));
        if text.chars().count() <= MAX_TEXT || n == 1 {
            return text;
        }
    }
    head
}

/// `alpha: 5 items stale > 3 days (oldest I-4 todo 9d)`.
#[must_use]
pub fn digest_text(scope: &str, list: &[Stale], escalate_ms: i64) -> String {
    let days = if escalate_ms % DAY_MS == 0 {
        format!(
            "{} day{}",
            escalate_ms / DAY_MS,
            if escalate_ms == DAY_MS { "" } else { "s" }
        )
    } else {
        age_text(escalate_ms)
    };
    let oldest = &list[0];
    format!(
        "{scope}: {} item{} stale > {days} (oldest {} {} {})",
        list.len(),
        plural(list.len()),
        oldest.id,
        oldest.status,
        age_text(oldest.quiet)
    )
}
