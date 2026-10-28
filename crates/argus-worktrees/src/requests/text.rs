//! The sentences of a removal request: what was asked, what was queued, how it ended.

use super::record::{Mode, RequestRec};

const SUMMARY_SENTENCE_MAX: usize = 140;
const SUMMARY_MAX: usize = 600;
const LABELS_SHOWN: usize = 3;

/// The text on one line: white space folded, cut to `max` characters.
pub(crate) fn one_line(text: &str, max: usize) -> String {
    let mut folded = String::new();
    for word in text.split_whitespace() {
        if !folded.is_empty() {
            folded.push(' ');
        }
        folded.push_str(word);
    }
    folded.chars().take(max).collect()
}

fn plural(n: usize, one: &str) -> String {
    format!("{n} {one}{}", if n == 1 { "" } else { "s" })
}

fn age_text(days: Option<f64>) -> String {
    match days {
        None => "the server's age limit".to_string(),
        Some(d) => {
            let s = if (d - 1.0).abs() < f64::EPSILON {
                ""
            } else {
                "s"
            };
            format!("{d} day{s}")
        }
    }
}

/// 90 is `90m`, 360 is `6h`, 2880 is `2d`: the shortest exact form.
pub fn idle_text(minutes: f64) -> String {
    let whole = |per: f64| (minutes / per).fract() == 0.0;
    if whole(1440.0) {
        format!("{}d", minutes / 1440.0)
    } else if whole(60.0) {
        format!("{}h", minutes / 60.0)
    } else {
        let fixed = format!("{minutes:.2}");
        format!("{}m", fixed.trim_end_matches('0').trim_end_matches('.'))
    }
}

fn idle_part(minutes: Option<f64>) -> String {
    match minutes.filter(|m| *m != 0.0) {
        Some(m) => format!(" and every idle clean worktree older than {}", idle_text(m)),
        None => String::new(),
    }
}

fn targets_text(rec: &RequestRec, branch: &str) -> String {
    let labels: Vec<&str> = rec
        .targets
        .iter()
        .take(LABELS_SHOWN)
        .map(|t| t.label.as_str())
        .collect();
    let more = rec.targets.len().saturating_sub(LABELS_SHOWN);
    let tail = if more > 0 {
        format!(" (+{more} more)")
    } else {
        String::new()
    };
    format!(
        "{}{branch}: {}{tail}",
        plural(rec.targets.len(), "worktree"),
        labels.join(", ")
    )
}

/// What the thread asked for, in plain words ("Argus will remove ...").
pub fn what_text(rec: &RequestRec) -> String {
    let branch = if rec.branch {
        " (and their branches where merged)"
    } else {
        ""
    };
    match rec.mode {
        Mode::Finished => format!(
            "every finished worktree of {} (merged, or stale and clean){} that its guards allow{branch}",
            rec.key,
            idle_part(rec.idle_minutes)
        ),
        Mode::Leftovers => format!(
            "every leftover folder of {} older than {} that its guards allow",
            rec.key,
            age_text(rec.older_than_days)
        ),
        Mode::Targets => targets_text(rec, branch),
    }
}

/// The answer to the thread that asked: nothing is removed by asking.
pub fn queued_text(rec: &RequestRec) -> String {
    format!(
        "queued {}: Argus will remove {}. Nothing is removed by this command; the result comes to your inbox (coord worktree requests shows it).",
        rec.id,
        what_text(rec)
    )
}

fn refused_part(rec: &RequestRec) -> String {
    let bad: Vec<&str> = rec
        .results
        .iter()
        .filter(|r| !r.ok)
        .map(|r| r.text.as_str())
        .collect();
    if bad.is_empty() {
        return String::new();
    }
    let mut sentences: Vec<String> = Vec::new();
    for text in bad.iter().map(|t| one_line(t, SUMMARY_SENTENCE_MAX)) {
        if !sentences.contains(&text) {
            sentences.push(text);
        }
    }
    let more = if sentences.len() > 3 {
        format!("; +{} more", sentences.len() - 3)
    } else {
        String::new()
    };
    let shown: Vec<&str> = sentences.iter().take(3).map(String::as_str).collect();
    format!(", refused {} ({}{more})", bad.len(), shown.join("; "))
}

fn skipped_part(rec: &RequestRec) -> String {
    match rec.skipped.as_ref().filter(|s| s.count > 0) {
        Some(s) => {
            let reasons: Vec<&str> = s.reasons.iter().take(2).map(String::as_str).collect();
            format!("; skipped {} ({})", s.count, reasons.join("; "))
        }
        None => String::new(),
    }
}

/// "worktree request R-3 partial: removed 14, refused 2 (sentence; sentence; +1 more); skipped 5 (younger than 1 day)".
pub fn summary_text(rec: &RequestRec) -> String {
    let skipped = skipped_part(rec);
    let body = if rec.results.is_empty() && skipped.is_empty() {
        rec.note
            .clone()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "nothing to remove".to_string())
    } else {
        format!("removed {}{}{skipped}", rec.removed(), refused_part(rec))
    };
    one_line(
        &format!(
            "worktree request {} {}: {body}",
            rec.id,
            rec.status.as_str()
        ),
        SUMMARY_MAX,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requests::record::{Outcome, SkippedSummary};

    fn outcome(ok: bool, text: &str) -> Outcome {
        Outcome {
            id: "x".into(),
            label: "x".into(),
            ok,
            text: text.into(),
            code: None,
        }
    }

    #[test]
    fn the_summary_counts_distinct_sentences_and_what_was_skipped() {
        let mut rec = RequestRec {
            id: "R-3".into(),
            status: crate::requests::record::RequestStatus::Partial,
            results: vec![outcome(true, "ok")],
            ..RequestRec::default()
        };
        for n in 0..5 {
            rec.results.push(outcome(false, &format!("no  way {n}")));
        }
        rec.results.push(outcome(false, "no way 0"));
        rec.skipped = Some(SkippedSummary {
            count: 5,
            reasons: vec!["5 younger than 1 day".into(), "1 b".into(), "1 c".into()],
        });
        assert_eq!(
            summary_text(&rec),
            "worktree request R-3 partial: removed 1, refused 6 (no way 0; no way 1; no way 2; +2 more); skipped 5 (5 younger than 1 day; 1 b)"
        );
    }

    #[test]
    fn nothing_to_remove_says_the_note() {
        let mut rec = RequestRec {
            id: "R-1".into(),
            ..RequestRec::default()
        };
        assert_eq!(
            summary_text(&rec),
            "worktree request R-1 queued: nothing to remove"
        );
        rec.note = Some("no leftover folder in the listing".into());
        assert!(summary_text(&rec).ends_with("queued: no leftover folder in the listing"));
    }

    #[test]
    fn the_ask_names_three_labels_and_counts_the_rest() {
        let mut rec = RequestRec {
            id: "R-2".into(),
            key: "alpha".into(),
            older_than_days: Some(0.25),
            ..RequestRec::default()
        };
        rec.targets = ["a", "b", "c", "d"]
            .iter()
            .map(|l| crate::requests::record::Target {
                id: (*l).into(),
                label: (*l).into(),
            })
            .collect();
        assert_eq!(what_text(&rec), "4 worktrees: a, b, c (+1 more)");
        rec.branch = true;
        rec.targets.truncate(1);
        assert_eq!(
            what_text(&rec),
            "1 worktree (and their branches where merged): a"
        );
        rec.mode = Mode::Leftovers;
        assert_eq!(
            what_text(&rec),
            "every leftover folder of alpha older than 0.25 days that its guards allow"
        );
        rec.older_than_days = Some(1.0);
        assert!(what_text(&rec).contains("older than 1 day that"));
        rec.older_than_days = None;
        assert!(what_text(&rec).contains("older than the server's age limit that"));
    }
}
